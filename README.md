<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# rv64_cpu: cleanups folded into the history

Checked the history of yzhou216/rv64_cpu in which the commits of its
`cleanup` branch were folded into the commits that introduce what they
change, before it replaced master.  The rewritten history sat on the
branch `fold-check` of rv64_cpu, whose CI checked its last commit.  On
`ubuntu-latest`, [fold/matrix.sh](fold/matrix.sh) ran every test program
of [fold/tests.dig](fold/tests.dig), which flake/programs.sh of the last
commit writes, on the processor of every commit in Digital and compared
the results with [fold/expected](fold/expected); nix flake check ran on
the commit that adds the flake, which has no CI of its own; and
record/record.py recorded Digital running the programs that each commit
with a fix makes pass, and all 24 on the last commit.

## Outcome

- Run 43415742554 (c7e5d3f): every recording reached DONE with the
  results that its commit should give: sll and srl by 66 on the commits
  that execute them, mul of -3 by 5, lui of 0x28000, beq, signed blt and
  bge, the link of jal past a jalr to an odd target, the sign extension of
  addiw, and all 24 programs on the last commit.  nix flake check passed
  on the commit that adds the flake.  The matrix differed from the
  prediction in fold/expected for 01-add-i and 13-mul only, which build a
  64-bit operand with slli by 32 and so pass only from the commit that
  decodes slli by funct6, not from the commits that execute addiw and mul.
- Run 43416579758 (86e04e2): with that correction in fold/expected, the
  matrix of every commit matched and nix flake check passed again.

rv64_cpu CI run 43415250648 passed on the last commit, 9d082d1, which
master and cleanup now point to.
