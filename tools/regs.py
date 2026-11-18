# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import stage
CTLW = 20
STAGES = [
    ('if-dr.dig', 0, 'IF/DR', [('PC4', 64), ('PC', 64), ('IW', 32)]),
    ('dr-ex.dig', 1, 'DR/EX', [('PC4', 64), ('PC', 64), ('CTL', CTLW), ('RD0', 64), ('RD1', 64), ('IW', 32)]),
    ('ex-mem.dig', 2, 'EX/MEM', [('CTL', CTLW), ('X', 64), ('D', 64), ('IW', 32)]),
    ('mem-wb.dig', 3, 'MEM/WB', [('CTL', CTLW), ('X', 64), ('DOut', 64), ('IW', 32)]),
]


def build(src):
    for fname, s, title, fields in STAGES:
        stage.stage_register(src, fname, s, title, fields)


if __name__ == '__main__':
    build(sys.argv[1])
