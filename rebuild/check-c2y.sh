#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0
#
# Checks every commit of c2y-rebase in REPO, which rebase c2y onto master:
# nix flake check of the commit, which builds rv64_emu with the flake and
# runs its checks, and make check with the RISC-V toolchain that PATH has
# as gcc and as, under qemu-riscv64.  Each commit gets one line in the log,
# followed by the failing output if there is any.
#
# Usage: check-c2y.sh <log>
set -uo pipefail
log=$1
work=$(mktemp -d)
ulimit -c 0

: >"$log"
for c in $(git -C "$REPO" rev-list --reverse origin/master..c2y-rebase); do
	flake="git+file://$REPO?rev=$c"
	if nix flake check -L "$flake" >"$work/flake.log" 2>&1; then
		cases=$(grep -o 'All [0-9]* cases passed' "$work/flake.log" | head -1)
		nix_result="nix flake check passed${cases:+, $cases}"
	else
		nix_result="NIX FLAKE CHECK FAILED"
	fi
	nix_warnings=$(nix log "$flake#rv64_emu" 2>/dev/null | grep -c 'warning:')

	rm -rf "$work/tree" && mkdir "$work/tree"
	git -C "$REPO" archive "$c" | tar -x -C "$work/tree"
	if ! make -s -C "$work/tree" CC=gcc AS=as >"$work/build.log" 2>&1; then
		make_result="MAKE FAILED"
	elif out=$(make -s -C "$work/tree" check CC=gcc AS=as RUN=qemu-riscv64 2>&1); then
		make_result="make check: $(printf '%s\n' "$out" | tail -1)"
	elif [ ! -f "$work/tree/tests/run.sh" ]; then
		make_result="make built it, no tests yet"
	else
		make_result="MAKE CHECK FAILED"
		printf '%s\n' "$out" | head -40 >"$work/check.log"
	fi
	make_warnings=$(grep -c 'warning:' "$work/build.log")

	printf '%s %-58s | %s (warnings: %s) | %s (warnings: %s)\n' \
		"$(git -C "$REPO" log -1 --format=%h "$c")" \
		"$(git -C "$REPO" log -1 --format=%s "$c" | cut -c1-58)" \
		"$nix_result" "$nix_warnings" "$make_result" "$make_warnings" >>"$log"
	case $nix_result in *FAILED*) grep -E 'error|differ|FAIL|^-|^\+' "$work/flake.log" | head -40 >>"$log" ;; esac
	case $make_result in *FAILED*) cat "$work/build.log" "$work/check.log" 2>/dev/null | head -40 >>"$log" ;; esac
done

# Fail if any commit failed a check
! grep -q 'FAILED' "$log"
