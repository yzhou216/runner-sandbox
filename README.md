<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# rv64_emu: history rebuild

This session rebuilt the history of
[yzhou216/rv64-emu](https://github.com/yzhou216/rv64-emu) on a hosted
runner instead of the laptop that asked for it:

- master, with the emulator and cache fixes of the c2y branch folded into
  the 2024 commits that introduced the bugs, and its two README commits
  folded into one;
- c2y, rebased onto that master and squashed from 72 commits into 15;
- `project04` renamed to `rv64_emu` in every file name, file, and commit
  message of both branches.

[rebuild/restructure.py](rebuild/restructure.py) builds every commit from
the trees of the original ones, with transforms that assert they match
exactly, and gives every c2y commit the expected test output of its own
build, run under `qemu-riscv64`.
[rebuild/verify.sh](rebuild/verify.sh) then builds every commit of both
branches with the RISC-V cross toolchain of nixpkgs `4fe25e1` (GCC 16.2.0)
and tests it.  The rebuilt branches came out as the `rebuilt` artifact, a git
bundle pushed to rv64-emu from the laptop.

## Runs

All on `ubuntu-latest`:

| Run | Result |
|-----|--------|
| [43085943626](https://github.com/yzhou216/runner-sandbox/actions/runs/43085943626) | Cancelled: the commit that adds the tests had no test cases |
| [43086190033](https://github.com/yzhou216/runner-sandbox/actions/runs/43086190033) | Cancelled to drop the course from the READMEs |
| [43086461862](https://github.com/yzhou216/runner-sandbox/actions/runs/43086461862) | Everything passed, but `verify.sh` exited with the status of its last comparison |
| [43091344991](https://github.com/yzhou216/runner-sandbox/actions/runs/43091344991) | Passed, with the same branches as the run before |

## Outcome

The rebuilt master is `7fbb1482d09f6bd8a9c5e25c60380dffa22eb463`, and
the rebuilt c2y is `877341615adb2ccd516d645407ab27b7f76ee107`.
Every commit of both branches built without warnings.  The emulated
results of the master commits match the native ones on more of the base
cases as the emulator grows, on all 41 from the commit that adds `sw` on,
and the master tip passes all 287 cases with every cache configuration.
Every c2y commit from the one that adds the tests on passes all of its
cases, 371 at first and 404 at the end, and the tree of the last one is
that of the original c2y with `project04` renamed and the course dropped
from the README.  A rebuild on the laptop gave the same master and,
README aside, the same first ten c2y trees, so the expected output does
not depend on the machine that generates it.
