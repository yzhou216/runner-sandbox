/*
 * SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
 *
 * SPDX-License-Identifier: 0BSD OR CC0-1.0
 */

/*
 * Runs one test program inside ReactOS with no error dialogs, which would
 * hold a crashed process open, and stops it after a number of seconds.
 * Its output, standard error, and exit status go to C:\results and to the
 * kernel debugger on COM1, which the host logs even if the disk is never
 * flushed.
 *
 *   runner <seconds> <name> <program> [argument...]
 */

#include <windows.h>
#include <stdarg.h>
#include <stdio.h>
#include <string.h>

static void say(const char *format, ...)
{
    char line[1024];
    va_list args;
    va_start(args, format);
    vsnprintf(line, sizeof line, format, args);
    va_end(args);
    OutputDebugStringA(line);
    fputs(line, stdout);
    fflush(stdout);
}

static HANDLE create(const char *name, const char *kind)
{
    char path[MAX_PATH];
    SECURITY_ATTRIBUTES inherit = { sizeof inherit, NULL, TRUE };
    snprintf(path, sizeof path, "C:\\results\\%s.%s.txt", name, kind);
    return CreateFileA(path, GENERIC_WRITE, FILE_SHARE_READ, &inherit,
                       CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
}

/* Lines of a result file, echoed to the debugger */
static void echo(const char *name, const char *kind)
{
    char path[MAX_PATH], line[512];
    snprintf(path, sizeof path, "C:\\results\\%s.%s.txt", name, kind);
    FILE *file = fopen(path, "r");
    if (!file)
        return;
    while (fgets(line, sizeof line, file)) {
        line[strcspn(line, "\r\n")] = '\0';
        say("ROSTEST %s %s: %s\n", name, kind, line);
    }
    fclose(file);
}

int main(int argc, char **argv)
{
    if (argc < 4) {
        say("usage: runner <seconds> <name> <program> [argument...]\n");
        return 2;
    }
    DWORD seconds = strtoul(argv[1], NULL, 10);
    const char *name = argv[2];
    char command[4096] = "";
    for (int i = 3; i < argc; i++) {
        strncat(command, "\"", sizeof command - strlen(command) - 1);
        strncat(command, argv[i], sizeof command - strlen(command) - 1);
        strncat(command, "\" ", sizeof command - strlen(command) - 1);
    }

    /* Inherited by the program, so that neither a failed import nor a crash
     * waits for someone to press OK */
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX |
                 SEM_NOOPENFILEERRORBOX);

    HANDLE out = create(name, "out"), err = create(name, "err");
    STARTUPINFOA startup = {
        .cb = sizeof startup,
        .dwFlags = STARTF_USESTDHANDLES,
        .hStdInput = GetStdHandle(STD_INPUT_HANDLE),
        .hStdOutput = out,
        .hStdError = err,
    };
    PROCESS_INFORMATION process;
    say("ROSTEST %s start: %s\n", name, command);
    if (!CreateProcessA(NULL, command, NULL, NULL, TRUE, 0, NULL, NULL,
                        &startup, &process)) {
        say("ROSTEST %s CreateProcess failed: %lu\n", name, GetLastError());
        return 1;
    }
    DWORD waited = WaitForSingleObject(process.hProcess, seconds * 1000);
    if (waited == WAIT_TIMEOUT) {
        TerminateProcess(process.hProcess, 0);
        WaitForSingleObject(process.hProcess, 10000);
    }
    DWORD status = 0;
    GetExitCodeProcess(process.hProcess, &status);
    CloseHandle(process.hThread);
    CloseHandle(process.hProcess);
    CloseHandle(out);
    CloseHandle(err);

    echo(name, "out");
    echo(name, "err");
    if (waited == WAIT_TIMEOUT)
        say("ROSTEST %s still running after %lu s, stopped\n", name, seconds);
    else
        say("ROSTEST %s exit 0x%08lx\n", name, status);
    return 0;
}
