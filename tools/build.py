# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Generate the pipelined rv64_cpu circuits into a source directory: build.py <src> [mutant]"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import regs, hazard, top
src = sys.argv[1]
mutant = sys.argv[2] if len(sys.argv) > 2 else ''
regs.build(src)
hazard.hazard_unit(src, mutant)
top.build(src, os.path.join(src, 'pipeline.dig'), mutant)
print('generated')
