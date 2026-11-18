# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Topology of the pipelined rv64_cpu for the planarity model."""
import sys, itertools
sys.path.insert(0, '/tmp/wire/plan')
from model import Model

CTL = ['PCsel', 'SZ', 'ST', 'LD', 'm2r', 'ALUOp', 'ALUSrcA', 'ALUSrcB', 'BrOp', 'RFW', 'WDsel']
GLOBAL = {'GCLK', 'EN', 'CLR'}


def cw_lr(left, right, top=(), bottom=()):
    """Clockwise pins of a box: top left to right, right top to bottom,
    bottom right to left, left bottom to top"""
    return list(top) + list(right) + list(reversed(bottom)) + list(reversed(left))


def stage(m, name, s, fields, nets=None):
    nets = nets or {}
    left = [(f'{f}_in', nets.get(f'{f}_{s}', f'{f}_{s}')) for f in fields]
    right = [(f'{f}_out', nets.get(f'{f}_{s + 1}', f'{f}_{s + 1}')) for f in fields]
    m.part(name, cw_lr(left, right))


def build(o):
    m = Model()
    p = m.part
    # IF
    p('pc', [('Q', 'PC_0'), ('EN', 'RUN'), ('CLK', 'GCLK'), ('D', 'NPC'), ('CLR', 'CLR')])
    p('add', [('s', 'PC4_0'), ('a', 'PC_0')])
    p('npc', [('out', 'NPC'), ('sel', 'FLUSH'), ('in_1', 'TARGET_2'), ('in_0', 'PC4_0')])
    p('addr', [('o', 'ADDR_0'), ('i', 'PC_0')])
    p('imem', [('IW', 'IW_0'), ('ADDR', 'ADDR_0')])
    p('run', [('out', 'RUN'), ('In_2', 'STALL'), ('In_1', 'EN')])
    p('clr1', [('out', 'CLR_1'), ('In_2', 'FLUSH'), ('In_1', 'CLR')])
    p('clr2', [('out', 'CLR_2'), ('In_3', 'STALL'), ('In_2', 'FLUSH'), ('In_1', 'CLR')])
    stage(m, 'ifdr', 0, o['ifdr'])
    n = len(o['ifdr'])
    m.parts['ifdr'] = m.parts['ifdr'][:n] + [('CLR', 'CLR_1'), ('EN', 'RUN')] + m.parts['ifdr'][n:]
    # DR
    p('idec', [(c, f'{c}_1') for c in CTL] + [('IW', 'IW_1')])
    p('merge', [('out', 'CTL_1')] + [(c, f'{c}_1') for c in reversed(CTL)], free=True)
    p('rdec1', [('RS1', 'RS1_1'), ('RS2', 'RS2_1'), ('IW', 'IW_1')])
    p('rf', [('RD0', 'RD0_1'), ('RD1', 'RD1_1'), ('WD', o.get('WD_rf', 'WD_4')), ('WEN', o.get('WEN_rf', 'WEN_4')),
             ('WREG', o.get('RD_rf', 'RD_4')), ('RR1', 'RS2_1'), ('RR0', 'RS1_1')])
    stage(m, 'drex', 1, o['drex'])
    m.parts['drex'] = m.parts['drex'][:len(o['drex'])] + [('CLR', 'CLR_2')] + m.parts['drex'][len(o['drex']):]
    # EX
    p('split2', o['split2'], free=True)
    p('rdec2', [('RS1', 'RS1_2'), ('RS2', 'RS2_2'), ('RD', 'RD_2'), ('IW', 'IW_2')])
    p('imm', [('i', 'IMMI'), ('s', 'IMMS'), ('b', 'IMMB'), ('u', 'IMMU'), ('j', 'IMMJ'), ('sh', 'SHAMT'), ('IW', 'IW_2')])
    p('mux0', [('out', 'FRD0'), ('sel', 'FWD0')] + list(reversed(o['mux0'])))
    p('mux1', [('out', 'FRD1'), ('sel', 'FWD1')] + list(reversed(o['mux1'])))
    p('srca', [('out', 'A_2'), ('sel', 'ALUSrcA_2'), ('in_1', 'PC_2'), ('in_0', 'FRD0')])
    p('srcb', [('out', 'B_2'), ('sel', 'ALUSrcB_2'), ('in_6', 'SHAMT'), ('in_5', 'IMMJ'), ('in_4', 'IMMU'),
               ('in_3', 'IMMB'), ('in_2', 'IMMS'), ('in_1', 'IMMI'), ('in_0', 'FRD1')])
    p('alu', [('R', 'R_2'), ('B', 'B_2'), ('A', 'A_2'), ('OP', 'ALUOp_2')])
    p('bcu', [('BrOp', 'BrOp_2'), ('taken', 'TAKEN_2'), ('A', 'FRD0'), ('B', 'FRD1')])
    p('br', [('=', 'BR_2'), ('a', 'PCsel_2')])
    p('jump', [('out', 'JUMP_2'), ('In_2', 'TAKEN_2'), ('In_1', 'BR_2')])
    p('hi', [('o', 'T_2'), ('i', 'R_2')])
    p('tgt', [('o', 'TARGET_2'), ('i', 'T_2')])
    p('hz', [(n, o.get('hz_' + n, n)) for n in ('RS1_1', 'RS2_1', 'RS1_2', 'RS2_2', 'RD_2', 'RFW_2', 'LD_2', 'RD_3',
                                               'RFW_3', 'RD_4', 'RFW_4', 'JUMP_2', 'FWD0', 'FWD1', 'STALL', 'FLUSH')],
      free=True)
    stage(m, 'exmem', 2, o['exmem'], {'D_2': 'FRD1'})
    # MEM
    p('split3', o['split3'], free=True)
    p('rdec3', [('RD', 'RD_3'), ('IW', 'IW_3')])
    p('maddr', [('o', 'MADDR'), ('i', 'R_3')])
    p('ram', [('D', 'MEM_3'), ('ld', 'LD_3'), ('C', 'GCLK'), ('str', 'ST_3'), ('D_in', 'SD_3'), ('A', 'MADDR')])
    p('load', [('SZ', 'SZ_3'), ('DOut', 'DOut_3'), ('DCur', o.get('DCUR_load', 'DCUR_3')), ('D', 'MEM_3'), ('A', 'R_3')])
    p('store', [('SZ', 'SZ_3'), ('DOut', 'SD_3'), ('D', 'D_3'), ('DCur', o.get('DCUR_store', 'DCUR_3')), ('A', 'R_3')])
    p('val', [('out', o.get('V_src', 'V_3')), ('sel', 'WDsel_3'), ('in_1', 'R_3'), ('in_0', 'PC4_3')])
    stage(m, 'memwb', 3, o['memwb'])
    # WB
    p('split4', o['split4'], free=True)
    p('rdec4', [('RD', 'RD_4'), ('IW', 'IW_4')])
    p('res', [('out', 'RES'), ('sel', 'm2r_4'), ('in_1', 'R_4'), ('in_0', 'DOut_4')])
    p('wd', [('out', o.get('WD_src', 'WD_4')), ('sel', 'WDsel_4'), ('in_1', 'RES'), ('in_0', 'PC4_4')])
    p('wen', [('out', o.get('WEN_src', 'WEN_4')), ('In_2', 'EN'), ('In_1', 'RFW_4')])
    p('done', [('=', 'DONE'), ('b', 'IW_4')])
    p('gclk', [('out', 'GCLK'), ('In_2', 'DONE')])
    return m


def run(o, cut=()):
    m = build(o)
    ok, cert = m.check(set(cut) | GLOBAL)
    if ok:
        return True, None
    return False, m.explain(cert)


def ok(o, cut=()):
    from model import planar
    return planar(build(o), set(cut) | GLOBAL)
