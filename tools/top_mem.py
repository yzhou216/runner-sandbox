# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Memory and write-back stages"""
from gen import path, tunnel, probe
from top_ex import split

L = {'shapeType': 'LAYOUT'}
UNIMP = 0xC0001073


def memory(c, exmem):
    memwb = c.add('mem-wb.dig', (4700, -120), **L)
    path(c, exmem.pin('CTL_3'), memwb.pin('CTL_3'))
    path(c, exmem.pin('X_3'), memwb.pin('X_3'))
    path(c, (3900, -100), (3900, -120))
    tunnel(c, (3900, -120), 'CTL_3', d='up', length=0)
    # Store: X is the address and D the data
    store = c.add('store.dig', (4020, 80), **L)
    path(c, (4000, -80), (4000, store.pin('A')[1]), store.pin('A'))
    d = exmem.pin('D_3')
    path(c, d, (3860, d[1]), (3860, store.pin('D')[1]), store.pin('D'))
    tunnel(c, store, 'DCur', 'DCUR_3', 'left')
    path(c, store.pin('SZ'), (store.pin('SZ')[0], 60))
    tunnel(c, (store.pin('SZ')[0], 60), 'SZ_3', d='up', length=0)
    ram = c.add('RAMDualPort', (4260, 100), AddrBits=7, Bits=64)
    maddr = c.add('Splitter', (4220, 100), **{'Input Splitting': '64', 'Output Splitting': '3-9'})
    path(c, (4220, -80), maddr.pin('0-63'))
    path(c, maddr.pin('3-9'), ram.pin('A'))
    path(c, store.pin('DOut'), ram.pin('D_in'))
    # Memory controls from below
    sp, out = split(c, (4080, 300), ['ST', 'LD', 'SZ', 'RFW'])
    tunnel(c, sp, '0-19', 'CTL_3', 'left')
    path(c, out['ST'], (4160, out['ST'][1]), (4160, ram.pin('str')[1]), ram.pin('str'))
    path(c, out['LD'], (4200, out['LD'][1]), (4200, ram.pin('ld')[1]), ram.pin('ld'))
    path(c, ram.pin('C'), (4180, ram.pin('C')[1]), (4180, 220))
    tunnel(c, (4180, 220), 'GCLK', d='down', length=0)
    tunnel(c, out['SZ'], 'SZ_3', d='right')
    tunnel(c, out['RFW'], 'RFW_3', d='right')
    load = c.add('load.dig', (4420, 100))
    path(c, ram.pin('D'), load.pin('D'))
    path(c, (4400, -80), (4400, load.pin('A')[1]), load.pin('A'))
    path(c, load.pin('SZ'), (load.pin('SZ')[0], 80))
    tunnel(c, (load.pin('SZ')[0], 80), 'SZ_3', d='up', length=0)
    o = load.pin('DOut')
    path(c, o, (4640, o[1]), (4640, memwb.pin('DOut_3')[1]), memwb.pin('DOut_3'))
    tunnel(c, load, 'DCur', 'DCUR_3', 'right')
    # The value forwarded from MEM is X
    path(c, (4600, -80), (4600, -60))
    tunnel(c, (4600, -60), 'V_3', d='down', length=0)
    # Destination register, for the hazard unit
    iw = exmem.pin('IW_3')
    path(c, iw, (3840, iw[1]), (3840, 400), (4680, 400), (4680, memwb.pin('IW_3')[1]), memwb.pin('IW_3'))
    probe(c, (4300, 400), (4300, 440), 'IW MEM')
    rdec = c.add('reg-decoder.dig', (3880, 300))
    path(c, (3840, 300), rdec.pin('IW'))
    tunnel(c, rdec, 'RD', 'RD_3', 'right')
    for n, net in (('EN', 'EN'), ('CLR', 'CLR'), ('CLK', 'GCLK')):
        tunnel(c, memwb, n, net, 'down')
    return {'memwb': memwb}


def writeback(c, memwb):
    sp, out = split(c, (4980, -260), ['RFW', 'm2r'])
    ctl = memwb.pin('CTL_4')
    path(c, ctl, (4960, ctl[1]), (4960, -260), sp.pin('0-19'))
    # X, or the loaded value for a load
    res = c.add('Multiplexer', (5180, -80), Bits=64, flipSelPos=True)
    path(c, memwb.pin('X_4'), res.pin('in_0'))
    dout = memwb.pin('DOut_4')
    path(c, dout, (5140, dout[1]), (5140, res.pin('in_1')[1]), res.pin('in_1'))
    inv = c.add('Not', (5120, out['m2r'][1]))
    path(c, out['m2r'], inv.pin('in'))
    path(c, inv.pin('out'), (res.pin('sel')[0], inv.pin('out')[1]), res.pin('sel'))
    tunnel(c, res, 'out', 'WD_4', 'right')
    # Write enable
    wen = c.add('And', (5080, -400))
    r = out['RFW']
    path(c, r, (5040, r[1]), (5040, wen.pin('In_2')[1]), wen.pin('In_2'))
    path(c, (5040, -320), (5020, -320))
    tunnel(c, (5020, -320), 'RFW_4', d='left', length=0)
    tunnel(c, wen, 'In_1', 'EN', 'left')
    tunnel(c, wen, 'out', 'WEN_4', 'right')
    # Destination register and the end of the program
    iw = memwb.pin('IW_4')
    path(c, iw, (4960, iw[1]), (4960, 300))
    c.add('Probe', (4960, 300), Label='IW WB', intFormat='hex')
    rdec = c.add('reg-decoder.dig', (5000, 100))
    path(c, (4960, 100), rdec.pin('IW'))
    tunnel(c, rdec, 'RD', 'RD_4', 'right')
    done = c.add('Comparator', (5000, 200), Bits=32)
    path(c, (4960, 200), done.pin('a'))
    c.add('Const', done.pin('b'), Value=UNIMP, Bits=32, intFormat='hex')
    e = done.pin('=')
    path(c, e, (5120, e[1]))
    c.add('Out', (5120, e[1]), Label='DONE')
    path(c, (5100, e[1]), (5100, e[1] + 40))
    tunnel(c, (5100, e[1] + 40), 'DONE', d='down', length=0)
