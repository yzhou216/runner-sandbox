#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0
#
# Build every commit of the rebuilt branches in REPO and test it.  Master
# has no test suite, so its commits run the base cases and count the cases
# whose emulated result matches the native one, and its tip runs them with
# every cache configuration too.  The c2y commits run make check.  With
# gcc and as for RISC-V and qemu-riscv64 on PATH.
#
# Usage: verify.sh <master log> <c2y log>
set -uo pipefail
master_log=$1 c2y_log=$2
work=$(mktemp -d)
ulimit -c 0

cat > "$work/base" <<'CASES'
fib_rec 0
fib_rec 1
fib_rec 2
fib_rec 10
fib_rec 20
get_bitseq 94116 12 15
get_bitseq 1023 4 7
get_bitseq 305419896 8 15
get_bitseq -1 0 31
int_to_str 1234 -o 10
int_to_str 1234 -o 16
int_to_str 255 -o 2
int_to_str -42 -o 10
int_to_str 0 -o 10
is_pal racecar
is_pal abba
is_pal abcd
is_pal a
max3 1 2 3
max3 3 2 1
max3 -5 -2 -9
max3 7 7 7
midpoint 2 8
midpoint 0 0
midpoint -10 10
midpoint 3 4
quadratic 2 3 4 5
quadratic -3 1 -2 7
quadratic 0 0 0 0
sort 5 3 9 1 7
sort 1
sort -3 10 0 -7 22 4 4 8
sort 1 2 3 4 5 6 7 8 9 10
str_to_int 1234
str_to_int 0x1f
str_to_int 0b1010
str_to_int 0xdeadbee
str_to_int 0
to_upper hello
to_upper Hello_World_123
to_upper already_UPPER
CASES

# Run each case and count those that exit 0 with matching Asm and Emu lines
agree() {
	local bin=$1 cases=$2 ok=0 n=0 c out
	while IFS= read -r c; do
		n=$((n + 1))
		# shellcheck disable=SC2086
		out=$(timeout -s KILL 10 qemu-riscv64 "$bin" $c 2>&1) || continue
		[ -n "$(printf '%s\n' "$out" | sed -n 's/^Asm://p')" ] || continue
		[ "$(printf '%s\n' "$out" | sed -n 's/^Asm://p')" = \
		  "$(printf '%s\n' "$out" | sed -n 's/^Emu://p')" ] && ok=$((ok + 1))
	done < "$cases"
	echo "$ok/$n agree"
}

line() {
	printf '%s %-60s %s\n' "$(git -C "$REPO" log -1 --format=%h "$1")" \
		"$(git -C "$REPO" log -1 --format=%s "$1" | cut -c1-60)" "$2"
}

: > "$master_log"
for c in $(git -C "$REPO" rev-list --reverse new-master); do
	rm -rf "$work/tree" && mkdir "$work/tree"
	git -C "$REPO" archive "$c" | tar -x -C "$work/tree"
	if ! make -s -C "$work/tree" > "$work/build.log" 2>&1; then
		line "$c" "BUILD FAILED" >> "$master_log"
		cat "$work/build.log" >> "$master_log"
		continue
	fi
	line "$c" "$(agree "$work/tree/rv64_emu" "$work/base") (warnings: $(grep -c 'warning:' "$work/build.log"))" >> "$master_log"
done
for f in '' '-a' '-dm 64 4' '-dm 16 1' '-dm 32 2' '-sa 64 4 4' '-sa 32 2 2'; do
	sed "s/^/${f:+$f }/" "$work/base"
done > "$work/matrix"
echo "master tip with every cache configuration: $(agree "$work/tree/rv64_emu" "$work/matrix")" >> "$master_log"

: > "$c2y_log"
for c in $(git -C "$REPO" rev-list --reverse new-master..new-c2y); do
	rm -rf "$work/tree" && mkdir "$work/tree"
	git -C "$REPO" archive "$c" | tar -x -C "$work/tree"
	if ! make -s -C "$work/tree" CC=gcc AS=as > "$work/build.log" 2>&1; then
		r="BUILD FAILED"
	elif [ ! -f "$work/tree/tests/run.sh" ]; then
		r="built, no tests yet"
	elif out=$(make -s -C "$work/tree" check CC=gcc AS=as RUN=qemu-riscv64 2>&1); then
		r=$(printf '%s\n' "$out" | tail -1)
	else
		r="CHECK FAILED"
		printf '%s\n' "$out" | head -40 > "$work/check.log"
	fi
	line "$c" "$r (warnings: $(grep -c 'warning:' "$work/build.log"))" >> "$c2y_log"
	[ "$r" = "CHECK FAILED" ] && cat "$work/check.log" >> "$c2y_log"
done
