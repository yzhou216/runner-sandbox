/*
 * SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
 *
 * SPDX-License-Identifier: 0BSD OR CC0-1.0
 */

/* Implicit TLS of an executable on ReactOS, with threads and a TLS callback
 * like the one of Rust's standard library */
#include <windows.h>
#include <stdio.h>

extern ULONG _tls_index;
static __thread volatile DWORD canary = 0x5a5a5a5a;
static __thread volatile DWORD zeroed;

static void say(const char *format, DWORD a, DWORD b, DWORD c)
{
    char line[256];
    snprintf(line, sizeof line, format, a, b, c);
    OutputDebugStringA(line);
    fputs(line, stdout);
    fflush(stdout);
}

static void NTAPI callback(PVOID module, DWORD reason, PVOID reserved)
{
    (void)module;
    (void)reserved;
    if (reason == DLL_THREAD_DETACH || reason == DLL_PROCESS_DETACH)
        say("tlsprobe: detach %lu thread %lu canary %08lx\n", reason,
            GetCurrentThreadId(), canary);
    if (reason == DLL_THREAD_DETACH && canary != GetCurrentThreadId())
        say("tlsprobe: MISMATCH thread %lu zeroed %lu index %lu\n",
            GetCurrentThreadId(), zeroed, _tls_index);
}

__attribute__((section(".CRT$XLB"), used))
const PIMAGE_TLS_CALLBACK probe_callback = callback;

static DWORD WINAPI thread(void *argument)
{
    (void)argument;
    if (canary != 0x5a5a5a5a || zeroed != 0)
        say("tlsprobe: BAD START thread %lu canary %08lx zeroed %lu\n",
            GetCurrentThreadId(), canary, zeroed);
    canary = GetCurrentThreadId();
    zeroed = 1;
    Sleep(100);
    return 0;
}

int main(void)
{
    HANDLE threads[4];
    say("tlsprobe: main %lu index %lu canary %08lx\n", GetCurrentThreadId(),
        _tls_index, canary);
    for (int i = 0; i < 4; i++)
        threads[i] = CreateThread(NULL, 0, thread, NULL, 0, NULL);
    WaitForMultipleObjects(4, threads, TRUE, INFINITE);
    say("tlsprobe: done %lu %lu %lu\n", 0, 0, 0);
    return 0;
}
