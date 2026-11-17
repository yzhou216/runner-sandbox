# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Generate the circuits of the pipelined rv64_cpu into a source directory:
build.py <src>"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gen

SRC = sys.argv[1]
MUTANT = os.environ.get('MUTANT', '')  # Breaks one hazard mechanism, to test the tests
UNIMP = 0xC0001073
CONTROLS = [('PCsel', 2), ('ALUSrcB', 3), ('ALUSrcA', 1), ('ALUOp', 4), ('RFW', 1), ('WDsel', 1),
            ('BrOp', 3), ('SZ', 2), ('ST', 1), ('LD', 1), ('m2r', 1)]
STAGES = [
    ('if-dr.dig', 0, 'IF/DR', [('IW', 32), ('PC', 64), ('PC4', 64)]),
    ('dr-ex.dig', 1, 'DR/EX', [('IW', 32), ('PC', 64), ('PC4', 64), ('RD0', 64), ('RD1', 64),
                               ('RS1', 5), ('RS2', 5), ('RD', 5)] + CONTROLS),
    ('ex-mem.dig', 2, 'EX/MEM', [('IW', 32), ('PC4', 64), ('R', 64), ('D', 64), ('RD', 5), ('RFW', 1),
                                 ('WDsel', 1), ('SZ', 2), ('ST', 1), ('LD', 1), ('m2r', 1)]),
    ('mem-wb.dig', 3, 'MEM/WB', [('IW', 32), ('PC4', 64), ('R', 64), ('DOut', 64), ('RD', 5),
                                 ('RFW', 1), ('WDsel', 1), ('m2r', 1)]),
]


def stage_register(fname, s, title, fields):
    c = gen.Circuit(SRC, {'Width': 6 if len(fields) < 5 else 10, 'Description': f'{title} pipeline register.  On a clock edge, '
                          'each field takes its input when EN or CLR is set, or zero when CLR is set.'})
    for i, (f, b) in enumerate(fields):
        y = i * 100
        inp = c.add('In', (0, y), Label=f'{f}_{s}', Bits=b)
        mux = c.add('Multiplexer', (100, y), Bits=b)
        c.wire(inp.pin('out'), mux.pin('in_0'))
        zero = c.add('Const', (60, y + 40), Value=0, Bits=b)
        c.wire(zero.pin('out'), mux.pin('in_1'))
        c.net(mux, 'sel', 'CLR')
        reg = c.add('Register', (200, y + 20), Bits=b, Label=f)
        c.wire(mux.pin('out'), reg.pin('D'))
        c.net(reg, 'C', 'CLK')
        c.net(reg, 'en', 'LOAD')
        out = c.add('Out', (340, y + 40), Label=f'{f}_{s + 1}', Bits=b)
        c.wire(reg.pin('Q'), out.pin('in'))
    y = len(fields) * 100
    for k, name in enumerate(('EN', 'CLR', 'CLK')):
        c.net(c.add('In', (0, y + 20 + 60 * k), Label=name), 'out', name)
    load = c.add('Or', (160, y + 20))
    c.nets(load, {'In_1': 'EN', 'In_2': 'CLR', 'out': 'LOAD'})
    c.write(os.path.join(SRC, fname))


def hazard_unit():
    c = gen.Circuit(SRC, {'Width': 6, 'Description':
        'Hazard unit.  FWD0 and FWD1 select the source of rs1 and rs2 in EX: 0 for the register '
        'file, 1 for the result in WB, and 2 for the result in MEM, which is newer.  STALL holds IF '
        'and DR for a cycle and sends a bubble to EX when DR needs the result of a load in EX.  FLUSH '
        'clears IF and DR when EX takes a branch or a jump.'})
    ins = [('RS1_1', 5), ('RS2_1', 5), ('RS1_2', 5), ('RS2_2', 5), ('RD_2', 5), ('RFW_2', 1), ('LD_2', 1),
           ('RD_3', 5), ('RFW_3', 1), ('RD_4', 5), ('RFW_4', 1), ('JUMP', 1)]
    for i, (n, b) in enumerate(ins):
        c.net(c.add('In', (0, i * 60), Label=n, Bits=b), 'out', n)
    # Which earlier result each source register reads
    for i, (rs, rd, net) in enumerate([('RS1_2', 'RD_3', 'S1_3'), ('RS1_2', 'RD_4', 'S1_4'),
                                       ('RS2_2', 'RD_3', 'S2_3'), ('RS2_2', 'RD_4', 'S2_4'),
                                       ('RS1_1', 'RD_2', 'S1_2'), ('RS2_1', 'RD_2', 'S2_2')]):
        cmp = c.add('Comparator', (300, i * 100), Bits=5)
        c.nets(cmp, {'a': rs, 'b': rd, '=': net})
    # Writes to x0 have no result to forward
    for i, (rd, net) in enumerate([('RD_2', 'NZ_2'), ('RD_3', 'NZ_3'), ('RD_4', 'NZ_4')]):
        y = 600 + i * 100
        cmp = c.add('Comparator', (300, y), Bits=5)
        c.net(cmp, 'a', rd)
        zero = c.add('Const', (260, y + 20), Value=0, Bits=5)
        c.wire(zero.pin('out'), cmp.pin('b'))
        c.net(cmp, '>', net)
    for i, (a, b, net) in enumerate([('S1_3', 'W_3', 'F0_3'), ('S1_4', 'W_4', 'F0_4'), ('S2_3', 'W_3', 'F1_3'),
                                     ('S2_4', 'W_4', 'F1_4'), ('RFW_2', 'NZ_2', 'W_2'), ('RFW_3', 'NZ_3', 'W_3'),
                                     ('RFW_4', 'NZ_4', 'W_4'), ('LD_2', 'W_2', 'L_2')]):
        if MUTANT == 'nox0' and net in ('W_3', 'W_4'):
            b = a
        g = c.add('And', (560, i * 100))
        c.nets(g, {'In_1': a, 'In_2': b, 'out': net})
    for i, (wb, mem, out) in enumerate([('F0_4', 'F0_3', 'FWD0'), ('F1_4', 'F1_3', 'FWD1')]):
        y = i * 160
        pe = c.add('PriorityEncoder', (820, y), **{'Selector Bits': 2})
        one = c.add('Const', (780, y), Value=1)
        c.wire(one.pin('out'), pe.pin('in0'))
        if MUTANT == 'wbfirst':
            wb, mem = mem, wb
        if MUTANT == 'noforward':
            wb = mem = 'ZERO'
        c.nets(pe, {'in1': wb, 'in2': mem})
        zero = c.add('Const', (780, y + 60), Value=0)
        c.wire(zero.pin('out'), pe.pin('in3'))
        o = c.add('Out', (980, y), Label=out, Bits=2)
        c.wire(pe.pin('num'), o.pin('in'))
    # A load in EX, whose result DR reads (stores set LD too, but not RFW)
    use = c.add('Or', (820, 340))
    c.nets(use, {'In_1': 'S1_2', 'In_2': 'S2_2', 'out': 'USE_2'})
    stall = c.add('And', (820, 440))
    c.nets(stall, {'In_1': 'L_2', 'In_2': 'USE_2', 'out': 'STALL'})
    flush = c.add('And', (820, 540), inverterConfig=['In_2'])
    c.nets(flush, {'In_1': 'JUMP', 'In_2': 'STALL'})
    o = c.add('Out', (980, 460), Label='STALL')
    c.net(o, 'in', 'ZERO' if MUTANT == 'nostall' else 'STALL')
    if MUTANT:
        c.net(c.add('Const', (1100, 700), Value=0), 'out', 'ZERO')
    o = c.add('Out', (980, 560), Label='FLUSH')
    if MUTANT == 'noflush':
        c.net(o, 'in', 'ZERO')
    else:
        c.wire(flush.pin('out'), o.pin('in'))
    c.write(os.path.join(SRC, 'hazard-unit.dig'))


def pipeline():
    c = gen.Circuit(SRC)
    # Inputs, outputs, clock, and the signals that steer the stages
    c.net(c.add('In', (-700, 0), Label='PROG', Bits=5), 'out', 'PROG')
    c.net(c.add('In', (-700, 60), Label='CLR'), 'out', 'CLR')
    c.net(c.add('In', (-700, 120), Label='EN'), 'out', 'EN')
    c.net(c.add('Clock', (-700, 180), Label='CLK'), 'C', 'CLK')
    for i, (label, net) in enumerate([('DONE', 'DONE'), ('A0', 'X10'), ('A1', 'X11'), ('A2', 'X12')]):
        c.net(c.add('Out', (-460, i * 60), Label=label, Bits=1 if label == 'DONE' else 64), 'in', net)
    gates = [('And', {'inverterConfig': ['In_2']}, {'In_1': 'CLK', 'In_2': 'DONE', 'out': 'GCLK'}),
             ('And', {'inverterConfig': ['In_2']}, {'In_1': 'EN', 'In_2': 'STALL', 'out': 'RUN'}),
             ('Or', {}, {'In_1': 'CLR', 'In_2': 'FLUSH', 'out': 'CLR_1'}),
             ('Or', {'Inputs': 3}, {'In_1': 'CLR', 'In_2': 'FLUSH', 'In_3': 'STALL', 'out': 'CLR_2'}),
             ('And', {}, {'In_1': 'RFW_4', 'In_2': 'EN', 'out': 'WEN_4'})]
    for i, (name, attrs, nets) in enumerate(gates):
        c.nets(c.add(name, (-640, 300 + i * 100), **attrs), nets)
    c.nets(c.add('Not', (-640, 820)), {'in': 'GCLK', 'out': 'RFCLK'})
    done = c.add('Comparator', (-640, 900), Bits=32)
    unimp = c.add('Const', (-680, 900), Value=UNIMP, Bits=32, intFormat='hex')
    c.wire(unimp.pin('out'), done.pin('a'))
    c.nets(done, {'b': 'IW_4', '=': 'DONE'})

    # IF: the program counter and the instruction memory
    c.text((0, -160), 'IF\ninstruction fetch')
    pc = c.add('64-bit-reg-clr.dig', (100, 0), Label='PC')
    c.nets(pc, {'CLR': 'CLR', 'D': 'NPC', 'CLK': 'GCLK', 'EN': 'RUN', 'Q': 'PC_0'})
    add = c.add('Add', (100, 160), Bits=64)
    four = c.add('Const', (60, 180), Value=4, Bits=64)
    zero = c.add('Const', (40, 200), Value=0)
    c.wire(four.pin('out'), add.pin('b'))
    c.wire(zero.pin('out'), add.pin('c_i'))
    c.nets(add, {'a': 'PC_0', 's': 'PC4_0'})
    npc = c.add('Multiplexer', (100, 300), Bits=64)
    c.nets(npc, {'sel': 'FLUSH', 'in_0': 'PC4_0', 'in_1': 'TARGET_2', 'out': 'NPC'})
    addr = c.add('Splitter', (100, 440), **{'Input Splitting': '64', 'Output Splitting': '2-9'})
    c.nets(addr, {'0-63': 'PC_0', '2-9': 'ADDR_0'})
    imem = c.add('inst-mem.dig', (100, 500))
    c.nets(imem, {'ADDR': 'ADDR_0', 'PROG': 'PROG', 'IW': 'IW_0'})

    def stage(fname, x, s, controls):
        part = c.add(fname, (x, 0))
        nets = {p: p for p in part.offsets if p not in ('EN', 'CLR', 'CLK')}
        nets.update(controls)
        c.nets(part, nets)
        return part

    stage('if-dr.dig', 500, 0, {'EN': 'RUN', 'CLR': 'CLR_1', 'CLK': 'GCLK'})

    # DR: decoding and the register file, which writes in the middle of the cycle
    c.text((900, -160), 'DR\ndecode and register read')
    dec = c.add('inst-decoder.dig', (1000, 0))
    c.nets(dec, {'IW': 'IW_1', **{f: f'{f}_1' for f, _ in CONTROLS}})
    regs = c.add('reg-decoder.dig', (1000, 300))
    c.nets(regs, {'IW': 'IW_1', 'RS1': 'RS1_1', 'RS2': 'RS2_1', 'RD': 'RD_1'})
    rf = c.add('reg-file.dig', (1000, 440))
    c.nets(rf, {'RR0': 'RS1_1', 'RR1': 'RS2_1', 'CLK': 'GCLK' if MUTANT == 'rfclk' else 'RFCLK', 'CLR': 'CLR', 'WEN': 'WEN_4',
                'WD': 'WD_4', 'WREG': 'RD_4', 'RD0': 'RD0_1', 'RD1': 'RD1_1',
                **{f'X{i}': f'X{i}' for i in range(32)}})
    stage('dr-ex.dig', 1500, 1, {'EN': 'EN', 'CLR': 'CLR_2', 'CLK': 'GCLK'})

    # EX: forwarding, the ALU, and the branch decision
    c.text((1900, -160), 'EX\nexecute')
    hz = c.add('hazard-unit.dig', (2000, 0))
    c.nets(hz, {p: p for p in hz.offsets if p not in ('JUMP',)})
    c.net(hz, 'JUMP', 'JUMP_2')
    for i, (sel, rd, out) in enumerate([('FWD0', 'RD0_2', 'FRD0_2'), ('FWD1', 'RD1_2', 'FRD1_2')]):
        y = 260 + i * 160
        mux = c.add('Multiplexer', (2000, y), Bits=64, **{'Selector Bits': 2})
        zero = c.add('Const', (1960, y + 60), Value=0, Bits=64)
        c.wire(zero.pin('out'), mux.pin('in_3'))
        c.nets(mux, {'sel': sel, 'in_0': rd, 'in_1': 'WD_4', 'in_2': 'V_3', 'out': out})
    srca = c.add('Multiplexer', (2300, 260), Bits=64)
    c.nets(srca, {'sel': 'ALUSrcA_2', 'in_0': 'FRD0_2', 'in_1': 'PC_2', 'out': 'A_2'})
    imm = c.add('imm-decoder.dig', (2000, 660))
    c.nets(imm, {'IW': 'IW_2', 'imm-i': 'IMMI_2', 'imm-s': 'IMMS_2', 'imm-b': 'IMMB_2',
                 'imm-u': 'IMMU_2', 'imm-j': 'IMMJ_2', 'shamt': 'SHAMT_2'})
    srcb = c.add('Multiplexer', (2300, 440), Bits=64, **{'Selector Bits': 3})
    zero = c.add('Const', (2260, 580), Value=0, Bits=64)
    c.wire(zero.pin('out'), srcb.pin('in_7'))
    c.nets(srcb, {'sel': 'ALUSrcB_2', 'in_0': 'FRD1_2', 'in_1': 'IMMI_2', 'in_2': 'IMMS_2', 'in_3': 'IMMB_2',
                  'in_4': 'IMMU_2', 'in_5': 'IMMJ_2', 'in_6': 'SHAMT_2', 'out': 'B_2'})
    alu = c.add('alu.dig', (2560, 300))
    c.nets(alu, {'A': 'A_2', 'B': 'B_2', 'OP': 'ALUOp_2', 'R': 'R_2'})
    bcu = c.add('bcu.dig', (2300, 760))
    c.nets(bcu, {'BrOp': 'BrOp_2', 'A': 'FRD0_2', 'B': 'FRD1_2', 'taken': 'TAKEN_2'})
    br = c.add('Comparator', (2560, 700), Bits=2)
    one = c.add('Const', (2520, 720), Value=1, Bits=2)
    c.wire(one.pin('out'), br.pin('b'))
    c.nets(br, {'a': 'PCsel_2', '=': 'BR_2'})
    jump = c.add('And', (2560, 820))
    c.nets(jump, {'In_1': 'BR_2', 'In_2': 'TAKEN_2', 'out': 'JUMP_2'})
    # Jump targets lose bit 0, which jalr requires
    hi = c.add('Splitter', (2560, 960), **{'Input Splitting': '64', 'Output Splitting': '1-63'})
    c.nets(hi, {'0-63': 'R_2', '1-63': 'T_2'})
    tgt = c.add('Splitter', (2720, 960), **{'Input Splitting': '1,63', 'Output Splitting': '64'})
    zero = c.add('Const', (2680, 960), Value=0)
    c.wire(zero.pin('out'), tgt.pin('0'))
    c.nets(tgt, {'1-63': 'T_2', '0-63': 'TARGET_2'})
    stage('ex-mem.dig', 2900, 2, {'D_2': 'FRD1_2', 'EN': 'EN', 'CLR': 'CLR', 'CLK': 'GCLK'})

    # MEM: the data memory, and the result that EX can take from here
    c.text((3300, -160), 'MEM\nmemory access')
    maddr = c.add('Splitter', (3400, 0), **{'Input Splitting': '64', 'Output Splitting': '3-9'})
    c.nets(maddr, {'0-63': 'R_3', '3-9': 'MADDR_3'})
    ram = c.add('RAMDualPort', (3400, 100), AddrBits=7, Bits=64)
    c.nets(ram, {'A': 'MADDR_3', 'D_in': 'SD_3', 'str': 'ST_3', 'C': 'GCLK', 'ld': 'LD_3', 'D': 'MEM_3'})
    load = c.add('load.dig', (3640, 120))
    c.nets(load, {'A': 'R_3', 'D': 'MEM_3', 'SZ': 'SZ_3', 'DOut': 'DOut_3', 'DCur': 'DCUR_3'})
    store = c.add('store.dig', (3400, 320))
    c.nets(store, {'D': 'D_3', 'DCur': 'DCUR_3', 'A': 'R_3', 'SZ': 'SZ_3', 'DOut': 'SD_3'})
    value = c.add('Multiplexer', (3640, 320), Bits=64)
    c.nets(value, {'sel': 'WDsel_3', 'in_0': 'PC4_3', 'in_1': 'R_3', 'out': 'V_3'})
    stage('mem-wb.dig', 3900, 3, {'EN': 'EN', 'CLR': 'CLR', 'CLK': 'GCLK'})

    # WB: the value that the register file takes
    c.text((4300, -160), 'WB\nwrite-back')
    res = c.add('Multiplexer', (4400, 0), Bits=64)
    c.nets(res, {'sel': 'm2r_4', 'in_0': 'DOut_4', 'in_1': 'R_4', 'out': 'RES_4'})
    wd = c.add('Multiplexer', (4400, 140), Bits=64)
    c.nets(wd, {'sel': 'WDsel_4', 'in_0': 'PC4_4', 'in_1': 'RES_4', 'out': 'WD_4'})

    # Displays: the instruction and the PC of every stage, and the registers
    for s, (stage_name, iw, pcnet) in enumerate([('IF', 'IW_0', 'PC_0'), ('DR', 'IW_1', 'PC_1'),
                                                  ('EX', 'IW_2', 'PC_2'), ('MEM', 'IW_3', None), ('WB', 'IW_4', None)]):
        x = s * 1000 + 100
        c.net(c.add('Probe', (x, 1300), Label=f'IW {stage_name}', intFormat='hex'), 'in', iw)
        if pcnet:
            c.net(c.add('Probe', (x, 1360), Label=f'PC {stage_name}', intFormat='hex'), 'in', pcnet)
    for i in range(32):
        x, y = (i // 8) * 400 + 100, 1480 + (i % 8) * 60
        c.net(c.add('Probe', (x, y), Label=f'X{i}'), 'in', f'X{i}')
    c.write(os.path.join(SRC, 'pipeline.dig'))


for fname, s, title, fields in STAGES:
    stage_register(fname, s, title, fields)
hazard_unit()
pipeline()
print('generated')
