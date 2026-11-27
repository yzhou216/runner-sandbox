// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: 0BSD OR CC0-1.0

/*
 * binfmt_misc interpreter that runs a program under QEMU with a large stack.
 * QEMU's linux-user takes an unlimited RLIMIT_STACK to mean its 8 MiB
 * default, while the GHC bootstrap chain sets the limit to unlimited for
 * programs that recurse deeply, so the stack size is passed explicitly.
 * Registered with the P flag, the interpreter gets the program's path, its
 * original argv[0], and then its arguments.
 */
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#ifndef QEMU
#error "QEMU must be the path to qemu-user"
#endif
#ifndef STACK
#error "STACK must be the guest stack size, as QEMU's -s takes it"
#endif

int main(int argc, char *argv[])
{
	char **args;
	int i;

	if (argc < 3) {
		fprintf(stderr, "%s: run as a binfmt_misc interpreter with the P flag\n",
			argv[0]);
		return 1;
	}

	args = calloc(argc + 4, sizeof(*args));
	if (!args) {
		perror("calloc");
		return 1;
	}
	args[0] = QEMU;
	args[1] = "-s";
	args[2] = STACK;
	args[3] = "-0";
	args[4] = argv[2];
	args[5] = argv[1];
	for (i = 3; i < argc; i++)
		args[i + 3] = argv[i];

	execv(QEMU, args);
	perror(QEMU);
	return 127;
}
