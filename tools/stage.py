# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Stage register: each field holds its value, takes the input, or clears.
SEL = CLR:EN picks the next value: 0 holds Q, 1 takes the input, 2 and 3
clear.  Two shared nets (SEL and CLK) reach every row without crossings."""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gen
from gen import path

H = 160          # Row height
SELX = -200      # SEL bus
CLKX = 560       # CLK bus


def stage_register(src, fname, s, title, fields, width=int(os.environ.get('SRW', 12))):
    c = gen.Circuit(src, {'Width': width, 'Description':
        f'{title} pipeline register.  On a clock edge, each field keeps its value, takes its input when EN '
        'is set, or clears when CLR is set.  CLR overrides EN.'})
    for i, (f, b) in enumerate(fields):
        y = i * H
        inp = c.add('In', (0, y + 20), Label=f'{f}_{s}', Bits=b)
        mux = c.add('Multiplexer', (100, y), Bits=b, **{'Selector Bits': 2})
        path(c, inp.pin('out'), mux.pin('in_1'))
        c.add('Const', mux.pin('in_2'), Value=0, Bits=b)
        c.add('Const', mux.pin('in_3'), Value=0, Bits=b)
        reg = c.add('Register', (200, y + 40), Bits=b, Label=f)
        path(c, mux.pin('out'), reg.pin('D'))
        c.add('Const', reg.pin('en'), Value=1)
        out = c.add('Out', (340, y + 60), Label=f'{f}_{s + 1}', Bits=b)
        q = reg.pin('Q')
        path(c, q, out.pin('in'))
        # Hold: Q back to in_0, over the top of the row
        path(c, (280, q[1]), (280, y - 20), (80, y - 20), (80, y), mux.pin('in_0'))
        # CLK from the bus on the right, SEL from the bus on the left
        path(c, (CLKX, y + 100), (160, y + 100), (160, reg.pin('C')[1]), reg.pin('C'))
        path(c, (SELX, y + 120), (120, y + 120), mux.pin('sel'))
    y = len(fields) * H
    path(c, (SELX, 120), (SELX, y + 40))
    path(c, (CLKX, 100), (CLKX, y + 40))
    # External pins at the bottom: EN, CLR, CLK from left to right
    en = c.add('In', (SELX - 120, y + 120), Label='EN', rotation=1)
    clr = c.add('In', (SELX - 60, y + 120), Label='CLR', rotation=1)
    sel = c.add('Splitter', (SELX - 20, y + 40), **{'Input Splitting': '1,1', 'Output Splitting': '2'})
    path(c, en.pin('out'), (en.pin('out')[0], sel.pin('0')[1]), sel.pin('0'))
    path(c, clr.pin('out'), (clr.pin('out')[0], sel.pin('1')[1]), sel.pin('1'))
    path(c, sel.pin('0-1'), (SELX, y + 40))
    clk = c.add('In', (CLKX, y + 120), Label='CLK', rotation=1)
    path(c, clk.pin('out'), (CLKX, y + 40))
    c.write(os.path.join(src, fname))
    return c


if __name__ == '__main__':
    src = sys.argv[1]
    stage_register(src, 'test-reg.dig', 1, 'DR/EX', [('PC4', 64), ('RD0', 64), ('CTL', 20), ('IW', 32)])
    print('ok')
