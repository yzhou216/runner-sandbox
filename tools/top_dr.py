# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Decode stage"""
from gen import path, tunnel, probe

CTL_SPLIT = '2,2,1,1,1,4,1,3,3,1,1'
L = {'shapeType': 'LAYOUT'}


def decode(c, ifdr):
    drex = c.add('dr-ex.dig', (1720, 0), **L)
    idec = c.add('inst-decoder.dig', (1000, 200), **L)
    merge = c.add('Splitter', (1160, 220), **{'Input Splitting': CTL_SPLIT, 'Output Splitting': '20'})
    for (pin, (x, y)), (mpin, (mx, my)) in zip(
            sorted(((p, idec.pin(p)) for p in idec.offsets if p != 'IW'), key=lambda kv: kv[1][1]),
            sorted(((p, merge.pin(p)) for p in merge.offsets if p != '0-19'), key=lambda kv: kv[1][1])):
        path(c, (x, y), (mx, my))
    out = merge.pin('0-19')
    path(c, out, (1220, out[1]), (1220, drex.pin('CTL_1')[1]), drex.pin('CTL_1'))
    # PC and PC4 pass along the top
    path(c, ifdr.pin('PC4_1'), drex.pin('PC4_1'))
    path(c, ifdr.pin('PC_1'), drex.pin('PC_1'))
    # Register file, which writes in the middle of the cycle
    rf = c.add('reg-file.dig', (1300, 480), **L)
    rdec = c.add('reg-decoder.dig', (1120, 580))
    path(c, rdec.pin('RS1'), rf.pin('RR0'))
    rs2 = rdec.pin('RS2')
    path(c, rs2, (1220, rs2[1]), (1220, rf.pin('RR1')[1]), rf.pin('RR1'))
    path(c, (1220, rf.pin('RR1')[1]), (1220, 680), (1200, 680))
    tunnel(c, (1200, 680), 'RS2_1', d='left', length=0)
    path(c, (1280, rf.pin('RR0')[1]), (1280, 540))
    tunnel(c, (1280, 540), 'RS1_1', d='up', length=0)
    tunnel(c, rf, 'WREG', 'RD_4', 'left')
    tunnel(c, rf, 'WEN', 'WEN_4', 'left')
    tunnel(c, rf, 'WD', 'WD_4', 'left')
    rfclk = c.add('Not', (rf.pin('CLK')[0] - 40, rf.pin('CLK')[1]))
    tunnel(c, rfclk, 'in', 'GCLK', 'left')
    tunnel(c, rf, 'CLR', 'CLR', 'left')
    rd0, rd1 = rf.pin('RD0'), rf.pin('RD1')
    path(c, rd0, (1440, rd0[1]), (1440, drex.pin('RD0_1')[1]), drex.pin('RD0_1'))
    path(c, rd1, (1460, rd1[1]), (1460, drex.pin('RD1_1')[1]), drex.pin('RD1_1'))
    # Register displays
    for i in range(32):
        p = rf.pin(f'X{i}')
        if i in (10, 11, 12):
            q = (p[0] + 40 + 80 * (i - 10), p[1])
            path(c, p, q)
            c.add('Out', q, Label=f'A{i - 10}', Bits=64)
        else:
            c.add('Probe', p)
    # The instruction word goes to both decoders and under the register file
    iw = ifdr.pin('IW_1')
    path(c, iw, (920, iw[1]), (920, idec.pin('IW')[1]), idec.pin('IW'))
    path(c, (920, idec.pin('IW')[1]), (920, rdec.pin('IW')[1]), rdec.pin('IW'))
    path(c, (920, rdec.pin('IW')[1]), (920, 1260), (1620, 1260), (1620, drex.pin("IW_1")[1]), drex.pin('IW_1'))
    probe(c, (1000, 1260), (1000, 1300), 'IW DR')
    # DR/EX takes no input during a stall and clears for a stall or a flush
    tunnel(c, drex, 'EN', 'EN', 'down')
    tunnel(c, drex, 'CLK', 'GCLK', 'down')
    clr2 = c.add('Or', (1740, 300), Inputs=3)
    tunnel(c, clr2, 'In_1', 'CLR', 'left')
    tunnel(c, clr2, 'In_2', 'FLUSH', 'left')
    tunnel(c, clr2, 'In_3', 'STALL', 'left')
    o = clr2.pin('out')
    p = drex.pin('CLR')
    path(c, o, (p[0], o[1]), p)
    return {'drex': drex, 'rf': rf}
