# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Top level of the pipelined rv64_cpu"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gen
from gen import path, tunnel
import top_if, top_dr, top_ex, top_mem


def inputs(c):
    clr = c.add('In', (-400, -100), Label='CLR')
    tunnel(c, clr, 'out', 'CLR', 'right')
    en = c.add('In', (-400, -40), Label='EN')
    tunnel(c, en, 'out', 'EN', 'right')
    clk = c.add('Clock', (-400, 40), Label='CLK')
    gate = c.add('And', (-300, 40), inverterConfig=['In_2'])
    path(c, clk.pin('C'), gate.pin('In_1'))
    tunnel(c, gate, 'In_2', 'DONE', 'left')
    tunnel(c, gate, 'out', 'GCLK', 'right')
    return None


def build(src, out, mutant=''):
    c = gen.Circuit(src)
    prog = inputs(c)
    f = top_if.fetch(c, prog)
    d = top_dr.decode(c, f['ifdr'])
    e = top_ex.execute(c, d['drex'], mutant)
    m = top_mem.memory(c, e['exmem'])
    top_mem.writeback(c, m['memwb'])
    for x, title in ((0, 'IF\ninstruction fetch'), (900, 'DR\ndecode and register read'), (2040, 'EX\nexecute'),
                     (3820, 'MEM\nmemory access'), (4940, 'WB\nwrite-back')):
        c.text((x, -460), title)
    c.write(out)
    return c
