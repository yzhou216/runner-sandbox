# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Fetch stage"""
from gen import path, tunnel, probe
L = {'shapeType': 'LAYOUT'}


def fetch(c, prog):
    ifdr = c.add('if-dr.dig', (600, 0), **L)
    pc = c.add('64-bit-reg-clr.dig', (200, 0), Label='PC')
    npc = c.add('Multiplexer', (80, 0), Bits=64)
    add = c.add('Add', (360, -100), Bits=64)
    c.add('Const', add.pin('b'), Value=4, Bits=64)
    c.add('Const', add.pin('c_i'), Value=0)
    addr = c.add('Splitter', (320, 280), **{'Input Splitting': '64', 'Output Splitting': '2-9'})
    imem = c.add('inst-mem.dig', (400, 240))
    path(c, npc.pin('out'), pc.pin('D'))
    tunnel(c, npc, 'in_1', 'TARGET_2', 'left')
    tunnel(c, npc, 'sel', 'FLUSH', 'down')
    s = add.pin('s')
    path(c, s, (440, s[1]), (440, -160), (-160, -160), (-160, 0), npc.pin('in_0'))
    path(c, (440, s[1]), (480, s[1]), (480, ifdr.pin('PC4_0')[1]), ifdr.pin('PC4_0'))
    q = pc.pin('Q')
    path(c, q, (300, q[1]), ifdr.pin('PC_0'))
    path(c, (300, q[1]), (300, add.pin('a')[1]), add.pin('a'))
    path(c, (300, q[1]), (300, addr.pin('0-63')[1]), addr.pin('0-63'))
    path(c, addr.pin('2-9'), imem.pin('ADDR'))
    iw = imem.pin('IW')
    path(c, iw, (560, iw[1]), (560, ifdr.pin('IW_0')[1]), ifdr.pin('IW_0'))
    probe(c, (560, 140), (580, 140), 'IW IF')
    # PC control: clear, clock, and enable unless stalled
    clrp = pc.pin('CLR')
    path(c, clrp, (180, clrp[1]), (180, -40))
    tunnel(c, (180, -40), 'CLR', d='up', length=0)
    tunnel(c, pc, 'CLK', 'GCLK', 'left')
    run = c.add('And', (440, 380), inverterConfig=['In_2'])
    tunnel(c, run, 'In_1', 'EN', 'left')
    tunnel(c, run, 'In_2', 'STALL', 'left')
    o = run.pin('out')
    en = pc.pin('EN')
    path(c, o, (o[0] + 20, o[1]), (o[0] + 20, 460), (180, 460), (180, en[1]), en)
    e = ifdr.pin('EN')
    path(c, (o[0] + 20, o[1]), (e[0], o[1]), e)
    # IF/DR clears for a flush
    clr1 = c.add('Or', (720, 200))
    path(c, clr1.pin('In_1'), (680, 200), (680, 300))
    tunnel(c, (680, 300), 'CLR', d='down', length=0)
    path(c, clr1.pin('In_2'), (700, 240), (700, 300))
    tunnel(c, (700, 300), 'FLUSH', d='down', length=0)
    o = clr1.pin('out')
    p = ifdr.pin('CLR')
    path(c, o, (800, o[1]), (800, 140), (p[0], 140), p)
    k = ifdr.pin('CLK')
    path(c, k, (k[0], 100), (k[0] + 40, 100))
    tunnel(c, (k[0] + 40, 100), 'GCLK', d='right', length=0)
    # Program select, from above the instruction memory
    prog = c.add('In', (380, 220), Label='PROG', Bits=5, rotation=3)
    path(c, prog.pin('out'), (380, imem.pin('PROG')[1]), imem.pin('PROG'))
    return {'ifdr': ifdr, 'imem': imem, 'prog': prog}
