#!/bin/sh
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Builds commits <first> through <last> of the original history of
# rv64_emu and of its folded counterpart, runs every case of tests/cases
# and of flake/agree.sh under qemu-riscv64 with each build, and fails
# unless both builds of every commit succeed, give the same warnings,
# and print the same output.  Before the commit that links the assembly
# first, the cache hits and misses depend on the size of the C objects
# linked before the assembly, so only those lines may differ there.
#
# Usage: compare.sh <repo> <first> <last> <outdir>
#
# CC and AS name the RISC-V compiler and assembler.  Wrappers named gcc
# and as add -Wall -Wextra and -march=rv64g, since the first commits run
# gcc and as by name and none before the c2y build assembles for rv64g,
# without which the assembler compresses instructions that the emulator
# does not decode.

set -u
repo=$1 first=$2 last=$3 out=$4
base=${BASE:-5d83c6211b5ef9a5feb9202445a1a15e2a0a0f08}
cc=${CC:-riscv64-unknown-linux-gnu-gcc}
as=${AS:-riscv64-unknown-linux-gnu-as}
mkdir -p "$out"
out=$(cd "$out" && pwd)
tmp=$(mktemp -d)
mkdir "$tmp/bin"
printf '#!/bin/sh\nexec %s -Wall -Wextra "$@"\n' "$cc" >"$tmp/bin/gcc"
printf '#!/bin/sh\nexec %s -march=rv64g "$@"\n' "$as" >"$tmp/bin/as"
chmod +x "$tmp/bin/gcc" "$tmp/bin/as"
PATH=$tmp/bin:$PATH

git -C "$repo" show "$base:tests/cases" >"$tmp/cases"
git -C "$repo" show "$base:flake/agree.sh" | sed -n '/^done <<CASES$/,/^CASES$/p' | sed '1d;$d' >>"$tmp/cases"
git -C "$repo" rev-list --reverse "$base" >"$tmp/orig"
git -C "$repo" rev-list --reverse refs/fold/master >"$tmp/fold"
[ "$(wc -l <"$tmp/orig")" -eq 43 ] && [ "$(wc -l <"$tmp/fold")" -eq 43 ] || exit 2

status=0
n=$first
while [ "$n" -le "$last" ]; do
	for kind in orig fold; do
		rev=$(sed -n "${n}p" "$tmp/$kind")
		dir=$tmp/$n-$kind
		mkdir "$dir"
		git -C "$repo" archive "$rev" | tar -x -C "$dir"
		if ! make -C "$dir" CC=gcc AS=as >"$out/$n-$kind.build" 2>&1; then
			echo "$n $kind: build failed"
			tail -20 "$out/$n-$kind.build"
			status=1
		fi
		sed -n 's/^[^ ]*:[0-9]*:[0-9]*: \(warning: .*\)/\1/p' "$out/$n-$kind.build" | sort >"$out/$n-$kind.warnings"
		set -f
		while IFS= read -r args || [ -n "$args" ]; do
			printf '$ %s\n' "$args"
			# shellcheck disable=SC2086 # Each case splits into arguments
			(cd "$dir" && timeout -s KILL 10 qemu-riscv64 ./rv64_emu $args 2>&1 </dev/null)
			printf '[exit %d]\n' $?
		done <"$tmp/cases" >"$out/$n-$kind.out"
		set +f
		if [ "$n" -lt 30 ]; then
			grep -v '^\(Hits\|Misses\|Misses (cold)\|Misses (hot)\|% Used\) ' "$out/$n-$kind.out" >"$out/$n-$kind.cmp"
		else
			cp "$out/$n-$kind.out" "$out/$n-$kind.cmp"
		fi
		rm -rf "$dir"
	done
	if ! diff -u "$out/$n-orig.warnings" "$out/$n-fold.warnings" >"$out/$n.warnings.diff"; then
		echo "$n: warnings differ"
		cat "$out/$n.warnings.diff"
		status=1
	fi
	if ! diff -u "$out/$n-orig.cmp" "$out/$n-fold.cmp" >"$out/$n.diff"; then
		echo "$n: output differs"
		head -40 "$out/$n.diff"
		status=1
	else
		echo "$n: same output for $(grep -c '^\$ ' "$out/$n-fold.out") cases, $(wc -l <"$out/$n-fold.warnings") warnings"
	fi
	n=$((n + 1))
done
rm -rf "$tmp"
exit $status
