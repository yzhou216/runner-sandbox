# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Hazard unit, drawn without tunnels or crossings"""
import os
from gen import Circuit, path

DESC = ('Hazard unit.  FWD0 and FWD1 select the source of rs1 and rs2 in EX.  FWD0 is 0 for the '
        'register file, 1 for the result in WB, and 2 for the result in MEM, which is newer; FWD1 '
        'counts down from 3 in the same order, for the reversed inputs of its mux.  STALL holds IF and '
        'DR for a cycle and sends a bubble to EX when DR reads the result of a load in EX.  FLUSH '
        'clears IF and DR when EX takes a branch or a jump.')


def hazard_unit(src, mutant=''):
    c = Circuit(src, {'Width': 8, 'Description': DESC})
    In = lambda pos, name, bits=5, **a: c.add('In', pos, Label=name, Bits=bits, **a)
    cmp = lambda pos: c.add('Comparator', pos, Bits=5)
    # Forwarding: each source register against the destinations in MEM and WB
    c1, c2, c3, c4 = (cmp((600, y)) for y in (0, 240, 480, 720))
    rs1 = In((380, 0), 'RS1_2')
    path(c, rs1.pin('out'), (400, 0), (400, 740), c4.pin('b'))
    path(c, (400, 0), c1.pin('a'))
    for top, bot, name, iny in ((c1.pin('b'), c2.pin('a'), 'RD_4', 80), (c2.pin('b'), c3.pin('a'), 'RS2_2', 360),
                                (c3.pin('b'), c4.pin('a'), 'RD_3', 560)):
        path(c, top, (580, top[1]), (580, bot[1]), bot)
        path(c, In((560, iny), name).pin('out'), (580, iny))
    ands = []
    for k, (ca, cb, rfw, nzy, name) in enumerate(((c1, c2, 'RFW_4', 140, 'W_4'), (c3, c4, 'RFW_3', 620, 'W_3'))):
        nz = cmp((700, nzy))
        path(c, (580, nzy), nz.pin('a'))
        c.add('Const', nz.pin('b'), Value=0, Bits=5)
        w = c.add('And', (800, nzy - 40))
        path(c, In((780, nzy - 40), rfw, 1).pin('out'), w.pin('In_1'))
        if mutant == 'nox0':
            c.add('Const', w.pin('In_2'), Value=1)
        else:
            path(c, nz.pin('>'), w.pin('In_2'))
        a1 = c.add('And', (900, ca.pin('=')[1]))
        a2 = c.add('And', (900, cb.pin('=')[1] - 40))
        path(c, ca.pin('='), a1.pin('In_1'))
        path(c, cb.pin('='), a2.pin('In_2'))
        o = w.pin('out')
        path(c, o, (880, o[1]), (880, a1.pin('In_2')[1]), a1.pin('In_2'))
        path(c, (880, o[1]), (880, a2.pin('In_1')[1]), a2.pin('In_1'))
        ands += [a1, a2]
    a14, a24, a23, a13 = ands
    zero = 'noforward' in mutant
    pe1 = c.add('PriorityEncoder', (1040, 220), **{'Selector Bits': 2})
    pe0 = c.add('PriorityEncoder', (1280, 20), **{'Selector Bits': 2})
    for pe in (pe0, pe1):
        c.add('Const', pe.pin('in0'), Value=1)
        c.add('Const', pe.pin('in3'), Value=0)
    wb, mem = ('in2', 'in1') if mutant == 'wbfirst' else ('in1', 'in2')
    if zero:
        for pe in (pe0, pe1):
            c.add('Const', pe.pin('in1'), Value=0)
            c.add('Const', pe.pin('in2'), Value=0)
    else:
        if mutant == 'wbfirst':
            c.wire(a24.pin('out'), pe1.pin(wb))
            c.wire(a23.pin('out'), pe1.pin(mem))
            c.wire(a14.pin('out'), pe0.pin(wb))
            c.wire(a13.pin('out'), pe0.pin(mem))
        else:
            path(c, a24.pin('out'), pe1.pin(wb))
            o = a23.pin('out')
            path(c, o, (1000, o[1]), (1000, pe1.pin(mem)[1]), pe1.pin(mem))
            path(c, a14.pin('out'), pe0.pin(wb))
            o = a13.pin('out')
            path(c, o, (1240, o[1]), (1240, pe0.pin(mem)[1]), pe0.pin(mem))
    inv = c.add('Not', (1160, 220), Bits=2)
    path(c, pe1.pin('num'), inv.pin('in'))
    o = inv.pin('out')
    path(c, o, (o[0], 400))
    c.add('Out', (o[0], 400), Label='FWD1', Bits=2, rotation=3)
    o = pe0.pin('num')
    path(c, o, (1400, o[1]), (1400, -60))
    c.add('Out', (1400, -60), Label='FWD0', Bits=2, rotation=1)
    # Load-use: a load in EX writes a register that DR reads (x0 never matches)
    nza, cma, cmb, nzb = (cmp((500, y)) for y in (860, 940, 1020, 1100))
    for nz in (nza, nzb):
        c.add('Const', nz.pin('b'), Value=0, Bits=5)
    path(c, nza.pin('a'), (460, 860), (460, 940), cma.pin('a'))
    path(c, In((420, 900), 'RS1_1').pin('out'), (460, 900))
    path(c, cma.pin('b'), (480, 960), (480, 1020), cmb.pin('a'))
    path(c, In((440, 980), 'RD_2').pin('out'), (480, 980))
    path(c, cmb.pin('b'), (460, 1040), (460, 1100), nzb.pin('a'))
    path(c, In((420, 1080), 'RS2_1').pin('out'), (460, 1080))
    ua = c.add('And', (640, 860))
    ub = c.add('And', (640, 1040))
    path(c, nza.pin('>'), ua.pin('In_1'))
    path(c, cma.pin('='), (600, 960), (600, 900), ua.pin('In_2'))
    path(c, cmb.pin('='), ub.pin('In_1'))
    path(c, nzb.pin('>'), (600, 1100), (600, 1080), ub.pin('In_2'))
    use = c.add('Or', (760, 880))
    path(c, ua.pin('out'), use.pin('In_1'))
    o = ub.pin('out')
    path(c, o, (720, o[1]), (720, use.pin('In_2')[1]), use.pin('In_2'))
    stall = c.add('And', (880, 900), Inputs=3)
    path(c, use.pin('out'), stall.pin('In_1'))
    path(c, In((820, 1120), 'RFW_2', 1).pin('out'), (840, 1120), (840, 920), stall.pin('In_2'))
    path(c, In((820, 1160), 'LD_2', 1).pin('out'), (860, 1160), (860, 940), stall.pin('In_3'))
    # A taken branch or jump flushes IF and DR, unless DR stalls
    br = c.add('Comparator', (900, 1200), Bits=2)
    path(c, In((840, 1200), 'PCsel_2', 2).pin('out'), br.pin('a'))
    c.add('Const', br.pin('b'), Value=1, Bits=2)
    jump = c.add('And', (1040, 1220))
    path(c, br.pin('='), jump.pin('In_1'))
    taken = c.add('In', (1080, 1320), Label='TAKEN', rotation=2)
    path(c, taken.pin('out'), (1020, 1320), (1020, 1260), jump.pin('In_2'))
    flush = c.add('And', (1160, 1200), inverterConfig=['In_1'])
    path(c, jump.pin('out'), flush.pin('In_2'))
    s = stall.pin('out')
    path(c, s, (1140, s[1]), (1140, flush.pin('In_1')[1]), flush.pin('In_1'))
    so = c.add('Out', (1340, 1420), Label='STALL', rotation=2)
    if mutant == 'nostall':
        c.add('Const', so.pin('in'), Value=0)
    else:
        path(c, (1140, 1000), (1360, 1000), (1360, 1420), so.pin('in'))
    fo = c.add('Out', (1220, 1460), Label='FLUSH', rotation=2)
    o = flush.pin('out')
    if mutant == 'noflush':
        c.add('Const', fo.pin('in'), Value=0)
    else:
        path(c, o, (1240, o[1]), (1240, 1460), fo.pin('in'))
    c.write(os.path.join(src, 'hazard-unit.dig'))
    return c
