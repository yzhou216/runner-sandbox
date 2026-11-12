#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0
#
# Builds every original commit of c2y in REPO and its rewritten
# counterpart in refs/rewrite/c2y, and compares the sections that hold
# code and data, which renames alone leave identical.  The builds use the
# CC, AS, and OBJCOPY of the flake's development shell, without Nixpkgs'
# hardening and without PIE, whose GOT is ordered by symbol name.  gcc and
# as on PATH lead there, since the early Makefiles run them by name.
#
# Usage: same-code.sh <log>
set -uo pipefail
log=$1
work=$(mktemp -d)
mkdir "$work/bin"
printf '#!/bin/sh\nexec %s -fno-pie -no-pie "$@"\n' "$CC" >"$work/bin/gcc"
printf '#!/bin/sh\nexec %s "$@"\n' "$AS" >"$work/bin/as"
chmod +x "$work/bin/gcc" "$work/bin/as"
export PATH="$work/bin:$PATH" NIX_HARDENING_ENABLE=

build() {
	rm -rf "${work:?}/${2:?}" && mkdir "$work/$2"
	git -C "$REPO" archive "$1" | tar -x -C "$work/$2"
	make -s -C "$work/$2" CC=gcc AS=as >"$work/$2.log" 2>&1
}

mapfile -t old < <(git -C "$REPO" rev-list --reverse origin/c2y)
mapfile -t new < <(git -C "$REPO" rev-list --reverse refs/rewrite/c2y)
[ "${#old[@]}" -eq "${#new[@]}" ] || { echo "commit counts differ" >"$log"; exit 1; }
: >"$log"
for i in "${!old[@]}"; do
	subject=$(git -C "$REPO" log -1 --format=%s "${new[$i]}" | cut -c1-58)
	if ! build "${old[$i]}" old || ! build "${new[$i]}" new; then
		result="BUILD FAILED"
		cat "$work/old.log" "$work/new.log" | head -20 >"$work/why"
	else
		result="identical"
		rm -f "$work"/old.* "$work"/new.*
		for s in .text .rodata .data .sdata .srodata; do
			$OBJCOPY -O binary --only-section=$s "$work/old/rv64_emu" "$work/old$s" 2>/dev/null
			$OBJCOPY -O binary --only-section=$s "$work/new/rv64_emu" "$work/new$s" 2>/dev/null
			cmp -s "$work/old$s" "$work/new$s" || result="DIFFERS in $s"
		done
	fi
	size=$(wc -c <"$work/new.text" 2>/dev/null || echo 0)
	printf '%2d %s %-58s %s (.text %s bytes)\n' "$((i + 1))" \
		"$(git -C "$REPO" log -1 --format=%h "${new[$i]}")" "$subject" "$result" "$size" >>"$log"
	case $result in BUILD*) cat "$work/why" >>"$log" ;; esac
done

# Fail unless every pair compiles to the same code and data
! grep -q -E 'DIFFERS|FAILED' "$log"
