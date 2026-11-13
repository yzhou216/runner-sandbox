#!/bin/sh
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Prints a line for every commit of master of the repository: its number,
# its hash, a + for every test program that passes and a . for every one
# that fails, and its subject.  Commits without the top-level circuit get
# dashes.  JAVA, JAR, and TESTS name Java, Digital.jar, and the circuit of
# test cases that progtests.py prints.
#
# Usage: matrix.sh <repo>

set -u
repo=$1
tmp=$(mktemp -d)
n=0
for c in $(git -C "$repo" rev-list --reverse master); do
	n=$((n + 1))
	short=$(git -C "$repo" rev-parse --short "$c")
	if ! git -C "$repo" cat-file -e "$c:src/rv64_cpu.dig" 2>/dev/null; then
		printf '%2d %s %s\n' $n "$short" "------------------------ (no circuit)"
		continue
	fi
	rm -rf "$tmp/w" && mkdir "$tmp/w" && git -C "$repo" archive "$c" | tar -x -C "$tmp/w"
	res=$(cd "$tmp/w/src" && "$JAVA" -Djava.awt.headless=true -cp "$JAR" CLI test -circ rv64_cpu.dig -tests "$TESTS" 2>&1)
	line=$(printf '%s\n' "$res" | awk '/^[0-9][0-9]-[a-z0-9-]*: (passed|failed)/ {printf "%s", ($2 == "passed" ? "+" : ".")}')
	printf '%2d %s %s %s\n' $n "$short" "${line:-error}" "$(git -C "$repo" log -1 --format=%s "$c")"
done
rm -rf "$tmp"
