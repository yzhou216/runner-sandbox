<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# rv64_emu: directory layout

This session rewrote the history of
[yzhou216/rv64_emu](https://github.com/yzhou216/rv64_emu) on a hosted
runner:

- the first three commits of master, the course starter code, its
  reformat in kernel style, and the removal of the rv_state typedef, became
  one initial commit by the author;
- the sources moved into `src` and the test programs into `progs` in every
  commit of master and c2y, with a Makefile that finds them through
  `VPATH`;
- every commit took its message from
  [rebuild/messages.py](rebuild/messages.py).

[rebuild/relayout.py](rebuild/relayout.py) checks that moving the files
and editing the Makefile are the only differences from the original
branches, and [rebuild/verify.sh](rebuild/verify.sh) builds every commit
of both branches with the RISC-V cross toolchain of nixpkgs `4fe25e1`
(GCC 16.2.0) and tests it under `qemu-riscv64`.

## Runs

All on `ubuntu-latest`:

| Run | Result |
|-----|--------|
| [43094747354](https://github.com/yzhou216/runner-sandbox/actions/runs/43094747354) | Rewrote, built, and tested both branches; the artifact upload failed, since the artifact service answered with HTML |
| [43098943720](https://github.com/yzhou216/runner-sandbox/actions/runs/43098943720) | Cancelled, since the log of the run before was enough |

## Outcome

The rewritten master is `4a960e00887303d33502f2fd4aa66bcedbb1c333`, with
25 commits, and the rewritten c2y is
`3dabf0d2a88aa23fab1ac9f283229fbc224162f3`, with 15 more.  A run of
relayout.py on the laptop produced the same commits, so they were pushed
to rv64_emu from there instead of from the missing bundle.

Every commit built without warnings.  The emulated results of the master
commits match the native ones on all 41 base cases from the commit that
adds `sw` on, and the master tip passes all 287 cases with every cache
configuration.  Every c2y commit from the one that adds the tests on
passes all of its cases, 371 at first and 404 at the end.
