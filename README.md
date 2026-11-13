<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# rv64_emu: counters past 2^31

Ran `rv64_emu -a -sa 64 4 2 fib_rec 40` from yzhou216/rv64_emu under
`qemu-riscv64` on `ubuntu-latest`, built by the flake of each revision,
before and after the counters of the instruction mix and of the cache
became 64 bits.  fib_rec 40 executes 3146022663 instructions, 19 times
fib(41) less 16, which no 32-bit int holds.

## Outcome

Run 43377103656 (f7f9ee6):

- 104c8ab, with int counters, printed `Instructions Executed  = -1148944633`
  and `References    = -1148944633`, the count less 2^32, along with
  percentages such as -100.88% that follow from it.  It took 9 minutes 53
  seconds.
- de32944, with 64-bit counters, printed 3146022663 for both, and 36.84%
  for R-type and I-type instructions.  It took 6 minutes 48 seconds.

Both returned 102334155 from the C version, the assembly, and the emulator.
