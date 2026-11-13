<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# rv64_cpu: Digital running the test programs

Recorded Digital 0.31 from Nixpkgs, with the runner's Java 21, running the
test programs of yzhou216/rv64_cpu on its single-cycle RV64 processor on an
Xvfb display of `ubuntu-latest`, to check in the GUI the circuit changes of
its `cleanup` branch that the headless tests of its flake had passed.
`record/record.py` enables the remote port of Digital through
`~/.digital.cfg`, marks the ROM of the program as program memory, sets PROG
and EN through the defaults of the inputs, starts the simulation over the
port, clocks it until DONE rises, and records the screen with ffmpeg.
`PROGRAMS` in the workflow picks the programs, and each run uploads a video
and the last frame of every program.

## Outcome

From run 43366716939 on, every program ended with DONE and the results that
the flake check of its commit expects.  The runs, with the commit of rv64_cpu
that each recorded:

- 43365871679 (8e04378, master 81d61f4) timed out: a connection that closes
  without a request ends the thread that serves the remote port, which the
  readiness probe of the script did.  The probe sends `measure` since.
- 43366257545 (a32ac0b, 81d61f4): every program reported DONE with A0 to A2
  at zero, since the defaults of PROG and EN had gone into the attribute
  `Default`, which In does not read; they go into `InDefault` since.
- 43366716939 (fbcd2c7, 81d61f4): 00-add-r, 18-max3, and 22-sort.
- 43367922902 (865c11a, 004fd24): lui gave 0x28000 once the ALU passed B
  through for it, with the pointer out of the frame.
- 43374154753 (0c0782b, 0d44e8a): all 24 programs.
- 43376408107 (93fc968, 56c8521): addiw sign-extending the low word of its
  sum, as 01-add-i ended with A0 5 and A1 0xFFFFFFFF80000000.
- 43378429874 (8e1c679, 3ba74ed): jalr clearing bit 0 of its target, and
  signed blt and bge, in 08-jal, 11-blt, and 12-bge.
- 43378897582 (d787fb6, 7807774): the widened register display, in which a
  16-digit value such as X11 of 01-add-i no longer runs into the next column.
- 43380950585 (2aac1ef, bfc91cf): jal and jalr through the reworked matchers
  of the decoder, in 08-jal, 09-j, and 20-is-pal.
