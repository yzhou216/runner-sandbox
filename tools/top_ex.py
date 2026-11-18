# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Execute stage"""
from gen import path, tunnel, probe

L = {'shapeType': 'LAYOUT'}
CTL = {'PCsel': '0-1', 'SZ': '2-3', 'ST': '4-4', 'LD': '5-5', 'm2r': '6-6', 'ALUOp': '7-10', 'ALUSrcA': '11-11',
       'ALUSrcB': '12-14', 'BrOp': '15-17', 'RFW': '18-18', 'WDsel': '19-19'}


def split(c, pos, fields):
    """Splitter that picks the named control fields from CTL"""
    sp = c.add('Splitter', pos, **{'Input Splitting': '20', 'Output Splitting': ','.join(CTL[f] for f in fields)})
    return sp, {f: sp.pin(CTL[f]) for f in fields}


def execute(c, drex, mutant=''):
    exmem = c.add('ex-mem.dig', (3580, -120), **L)
    # PC and PC4 go by name to the operand and result muxes
    tunnel(c, drex, 'PC4_2', 'PC4_2', 'right')
    tunnel(c, drex, 'PC_2', 'PC_2', 'right')
    # Controls along the top
    ctl = drex.pin('CTL_2')
    path(c, ctl, (2080, ctl[1]), (2080, -100), exmem.pin('CTL_2'))
    sp, out = split(c, (2140, -40), ['WDsel', 'ALUOp', 'ALUSrcA', 'ALUSrcB', 'BrOp'])
    path(c, (2120, -100), (2120, -40), sp.pin('0-19'))
    tunnel(c, out['ALUSrcB'], 'ALUSrcB_2', d='right')
    tunnel(c, out['BrOp'], 'BrOp_2', d='right')
    path(c, (2300, -100), (2300, -120))
    tunnel(c, (2300, -120), 'CTL_2', d='up', length=0)
    # Forwarding muxes; the second one takes its inputs in reverse order
    mux0 = c.add('Multiplexer', (2240, 200), Bits=64, **{'Selector Bits': 2})
    mux1 = c.add('Multiplexer', (2240, 700), Bits=64, **{'Selector Bits': 2, 'flipSelPos': True})
    c.add('Const', mux0.pin('in_3'), Value=0, Bits=64)
    c.add('Const', mux1.pin('in_0'), Value=0, Bits=64)
    rd0, rd1, iw = drex.pin('RD0_2'), drex.pin('RD1_2'), drex.pin('IW_2')
    path(c, rd0, (2020, rd0[1]), (2020, mux0.pin('in_0')[1]), mux0.pin('in_0'))
    path(c, rd1, (2000, rd1[1]), (2000, mux1.pin('in_3')[1]), mux1.pin('in_3'))
    path(c, mux0.pin('in_1'), (2160, mux0.pin('in_1')[1]), (2160, mux1.pin('in_2')[1]), mux1.pin('in_2'))
    path(c, mux0.pin('in_2'), (2180, mux0.pin('in_2')[1]), (2180, mux1.pin('in_1')[1]), mux1.pin('in_1'))
    path(c, (2160, 440), (2140, 440))
    tunnel(c, (2140, 440), 'WD_4', d='left', length=0)
    tunnel(c, (2180, 320), 'V_3', d='right')
    # Hazard unit between the forwarding muxes, with the register numbers and controls it reads
    hz = c.add('hazard-unit.dig', (2560, 340), **L)
    rdec = c.add('reg-decoder.dig', (2320, 360))
    path(c, rdec.pin('RS1'), hz.pin('RS1_2'))
    p = rdec.pin('RS2')
    path(c, p, (2440, p[1]), (2440, hz.pin('RS2_2')[1]), hz.pin('RS2_2'))
    p = rdec.pin('RD')
    path(c, p, (2420, p[1]), (2420, hz.pin('RD_2')[1]), hz.pin('RD_2'))
    tunnel(c, rdec, 'IW', 'IW_2', 'left')
    gsp, gout = split(c, (2500, hz.pin('RFW_2')[1]), ['RFW', 'LD', 'PCsel'])
    for f in ('RFW', 'LD', 'PCsel'):
        path(c, gout[f], hz.pin(f + '_2'))
    tunnel(c, gsp, '0-19', 'CTL_2', 'left')
    for n in ('RD_4', 'RFW_4', 'RD_3', 'RFW_3', 'RS1_1', 'RS2_1', 'STALL', 'FLUSH'):
        tunnel(c, hz, n, n, 'left')
    f0, s0 = hz.pin('FWD0'), mux0.pin('sel')
    path(c, f0, (f0[0], 300), (s0[0], 300), s0)
    f1, s1 = hz.pin('FWD1'), mux1.pin('sel')
    path(c, f1, (f1[0], 660), (s1[0], 660), s1)
    # Branch unit, turned so that its operands come from the right
    fr0, fr1 = mux0.pin('out'), mux1.pin('out')
    t = hz.pin('TAKEN')
    bcu = c.add('bcu.dig', (t[0] + 140, t[1] + 140), rotation=2)
    path(c, (2900, fr0[1]), (2900, bcu.pin('A')[1]), bcu.pin('A'))
    path(c, (2900, fr1[1]), (2900, bcu.pin('B')[1]), bcu.pin('B'))
    path(c, bcu.pin('taken'), t)
    b = bcu.pin('BrOp')
    path(c, b, (b[0], b[1] + 20))
    tunnel(c, (b[0], b[1] + 20), 'BrOp_2', d='down', length=0)
    # Operands
    srca = c.add('Multiplexer', (3040, fr0[1]), Bits=64, flipSelPos=True)
    srcb = c.add('Multiplexer', (3040, fr1[1]), Bits=64, **{'Selector Bits': 3})
    path(c, fr0, srca.pin('in_0'))
    path(c, fr1, srcb.pin('in_0'))
    tunnel(c, srca, 'in_1', 'PC_2', 'left')
    path(c, out['ALUSrcA'], (srca.pin('sel')[0], out['ALUSrcA'][1]), srca.pin('sel'))
    tunnel(c, srcb, 'sel', 'ALUSrcB_2', 'down')
    c.add('Const', srcb.pin('in_7'), Value=0, Bits=64)
    imm = c.add('imm-decoder.dig', (2920, srcb.pin('in_1')[1]))
    for a, b in zip(('imm-i', 'imm-s', 'imm-b', 'imm-u', 'imm-j', 'shamt'), range(1, 7)):
        path(c, imm.pin(a), srcb.pin(f'in_{b}'))
    path(c, (2980, fr1[1]), (2980, fr1[1] - 20))
    tunnel(c, (2980, fr1[1] - 20), 'D_2', d='up', length=0)
    # ALU and the result: the ALU output, or PC+4 for a jump
    alu = c.add('alu.dig', (3220, 500), **L)
    a, b = srca.pin('out'), srcb.pin('out')
    path(c, a, (3140, a[1]), (3140, alu.pin('A')[1]), alu.pin('A'))
    path(c, b, (3160, b[1]), (3160, alu.pin('B')[1]), alu.pin('B'))
    op = alu.pin('OP')
    path(c, out['ALUOp'], (3180, out['ALUOp'][1]), (3180, op[1]), op)
    xmux = c.add('Multiplexer', (3380, 500), Bits=64, flipSelPos=True)
    r = alu.pin('R')
    path(c, r, xmux.pin('in_1'))
    tunnel(c, xmux, 'in_0', 'PC4_2', 'left')
    path(c, out['WDsel'], (xmux.pin('sel')[0], out['WDsel'][1]), xmux.pin('sel'))
    x = xmux.pin('out')
    path(c, x, (3460, x[1]), (3460, exmem.pin('X_2')[1]), exmem.pin('X_2'))
    tunnel(c, exmem, 'D_2', 'D_2', 'left')
    # Jump target: the ALU result with bit 0 cleared
    hi = c.add('Splitter', (3340, 600), **{'Input Splitting': '64', 'Output Splitting': '1-63'})
    tgt = c.add('Splitter', (3420, 580), **{'Input Splitting': '1,63', 'Output Splitting': '64'})
    path(c, (3320, r[1]), (3320, 600), hi.pin('0-63'))
    path(c, hi.pin('1-63'), tgt.pin('1-63'))
    c.add('Const', tgt.pin('0'), Value=0)
    tunnel(c, tgt, '0-63', 'TARGET_2', 'right')
    # Instruction word under everything to EX/MEM, and to the immediate decoder
    e = exmem.pin('IW_2')
    path(c, iw, (1980, iw[1]), (1980, 960), (3560, 960), (3560, e[1]), e)
    path(c, (2880, 960), (2880, imm.pin('IW')[1]), imm.pin('IW'))
    probe(c, (2600, 960), (2600, 1000), 'IW EX')
    path(c, (2400, 960), (2400, 980))
    tunnel(c, (2400, 980), 'IW_2', d='down', length=0)
    for n, net in (('EN', 'EN'), ('CLR', 'CLR'), ('CLK', 'GCLK')):
        tunnel(c, exmem, n, net, 'down')
    return {'exmem': exmem}
