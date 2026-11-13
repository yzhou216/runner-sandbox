#!/bin/sh
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

# Runs every test program on the processor in Digital and fails unless it
# reaches the unimp at its end, which raises DONE, with the result that it
# documents.  Each line below names a program, whose number PROG takes,
# and the values of A0, A1, and A2, or X for any value.
#
# Usage: programs.sh <java> <Digital.jar> <src>

set -eu
java=$1 jar=$2 src=$3
tests=$(mktemp)
{
	printf '<?xml version="1.0" encoding="utf-8"?>\n<circuit>\n  <version>2</version>\n'
	printf '  <attributes/>\n  <visualElements>\n'
	while read -r name a0 a1 a2; do
		prog=${name%%-*}
		prog=${prog#0}
		cat <<EOF
    <visualElement>
      <elementName>Testcase</elementName>
      <elementAttributes>
        <entry>
          <string>Label</string>
          <string>$name</string>
        </entry>
        <entry>
          <string>Testdata</string>
          <testData>
            <dataString>PROG CLR EN CLK DONE A0 A1 A2
$prog 1 0 C X X X X
$prog 0 1 0 X X X X
repeat(3000) $prog 0 1 C X X X X
$prog 0 1 0 1 $a0 $a1 $a2</dataString>
          </testData>
        </entry>
      </elementAttributes>
      <pos x="0" y="0"/>
    </visualElement>
EOF
	done <<PROGRAMS
00-add-r X X 3
01-add-i X X 3
02-sll-r 4 X X
03-sll-i 8 X X
04-srl-r 4 X X
05-srl-i 8 X X
06-sd-ld 99 X X
07-sw-lw 99 X X
08-jal 3 X X
09-j 1 X X
10-beq 1 X X
11-blt 1 X X
12-bge 2 X X
13-mul 9 X X
14-lui 0x1000 X X
15-sb-lb 99 X X
16-quadratic 25 X X
17-get-bitseq 6 X X
18-max3 2 X X
19-fib-rec 55 X X
20-is-pal 1 X X
21-to-upper 0x464F4F4241520000 X X
22-sort 0xC0000000A 0x800000006 0x400000002
23-str-to-int 0xFFFF X X
PROGRAMS
	printf '  </visualElements>\n  <wires/>\n  <measurementOrdering/>\n</circuit>\n'
} >"$tests"
cd "$src"
"$java" -Djava.awt.headless=true -cp "$jar" CLI test -circ rv64_cpu.dig -tests "$tests"
