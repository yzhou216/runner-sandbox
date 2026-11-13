#!/bin/sh
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Prints a line for each of the commits first to last of the checked-out
# branch of rv64_cpu, numbered from its root, that has the top-level
# circuit: its number and a + for every test program of tests.dig that
# passes in Digital, or a . for every one that fails.  JAVA and JAR name
# Java and Digital.jar.
#
# Usage: matrix.sh <rv64_cpu> <first> <last> <tests.dig>

set -u
repo=$1 first=$2 last=$3 tests=$4
tmp=$(mktemp -d)
n=0
for c in $(git -C "$repo" rev-list --reverse HEAD); do
	n=$((n + 1))
	if [ "$n" -lt "$first" ] || [ "$n" -gt "$last" ]; then
		continue
	fi
	if ! git -C "$repo" cat-file -e "$c:src/rv64_cpu.dig" 2>/dev/null; then
		continue
	fi
	rm -rf "$tmp/w" && mkdir "$tmp/w" && git -C "$repo" archive "$c" | tar -x -C "$tmp/w"
	res=$(cd "$tmp/w/src" && "$JAVA" -Djava.awt.headless=true -cp "$JAR" CLI test -circ rv64_cpu.dig -tests "$tests" 2>&1)
	line=$(printf '%s\n' "$res" | awk '/^[0-9][0-9]-[a-z0-9-]*: (passed|failed)/ { printf "%s", ($2 == "passed" ? "+" : ".") }')
	printf '%2d %s\n' "$n" "${line:-error}"
	printf '%2d %s %s\n' "$n" "$(git -C "$repo" rev-parse --short "$c")" "$(git -C "$repo" log -1 --format=%s "$c")" >&2
done
rm -rf "$tmp"
