# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Print a Digital circuit whose test cases run every test program of
rv64_cpu for 3000 cycles and check that it raised DONE with the result
that it documents.  Unlike flake/programs.sh, it works on every commit
that has the top-level circuit, including those before the flake."""
from xml.sax.saxutils import escape

RESULTS = [
    ('00-add-r', {'A2': '3'}), ('01-add-i', {'A2': '3'}),
    ('02-sll-r', {'A0': '4'}), ('03-sll-i', {'A0': '8'}),
    ('04-srl-r', {'A0': '4'}), ('05-srl-i', {'A0': '8'}),
    ('06-sd-ld', {'A0': '99'}), ('07-sw-lw', {'A0': '99'}),
    ('08-jal', {'A0': '3'}), ('09-j', {'A0': '1'}),
    ('10-beq', {'A0': '1'}), ('11-blt', {'A0': '1'}),
    ('12-bge', {'A0': '2'}), ('13-mul', {'A0': '9'}),
    ('14-lui', {'A0': '0x1000'}), ('15-sb-lb', {'A0': '99'}),
    ('16-quadratic', {'A0': '25'}), ('17-get-bitseq', {'A0': '6'}),
    ('18-max3', {'A0': '2'}), ('19-fib-rec', {'A0': '55'}),
    ('20-is-pal', {'A0': '1'}), ('21-to-upper', {'A0': '0x464F4F4241520000'}),
    ('22-sort', {'A0': '0xC0000000A', 'A1': '0x800000006', 'A2': '0x400000002'}),
    ('23-str-to-int', {'A0': '0xFFFF'}),
]


def case(label, data):
    return f'''    <visualElement>
      <elementName>Testcase</elementName>
      <elementAttributes>
        <entry>
          <string>Label</string>
          <string>{escape(label)}</string>
        </entry>
        <entry>
          <string>Testdata</string>
          <testData>
            <dataString>{escape(data)}</dataString>
          </testData>
        </entry>
      </elementAttributes>
      <pos x="0" y="0"/>
    </visualElement>
'''


cases = []
for p, (name, want) in enumerate(RESULTS):
    regs = ' '.join(want.get(r, 'X') for r in ('A0', 'A1', 'A2'))
    cases.append(case(name, f'PROG CLR EN CLK DONE A0 A1 A2\n{p} 1 0 C X X X X\n{p} 0 1 0 X X X X\n'
                            f'repeat(3000) {p} 0 1 C X X X X\n{p} 0 1 0 1 {regs}\n'))
print('<?xml version="1.0" encoding="utf-8"?>\n<circuit>\n  <version>2</version>\n'
      '  <attributes/>\n  <visualElements>\n' + ''.join(cases) +
      '  </visualElements>\n  <wires/>\n  <measurementOrdering/>\n</circuit>', end='\n')
