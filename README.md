<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# rv64_emu: GPLv3

This session rewrote the history of
[yzhou216/rv64_emu](https://github.com/yzhou216/rv64_emu) on a hosted
runner:

- the commit that drops the remaining typedefs folded into the initial
  commit, with the twelve commits between them rebased onto it, and the
  `rv_format` typedef that it missed became a plain enum, so that no commit
  has a typedef;
- every commit of master and c2y has the GPLv3 instead of the MIT license,
  and its README opens with a tagline instead of a sentence.

[rebuild/relicense.py](rebuild/relicense.py) checks that the rebased
commits end with the tree of the typedef commit, so that every later tree
is unchanged apart from the license, the README, and `rv_format`.
[rebuild/verify.sh](rebuild/verify.sh) builds every commit of both
branches with the RISC-V cross toolchain of nixpkgs `4fe25e1` (GCC 16.2.0)
and tests it under `qemu-riscv64`.

## Runs

All on `ubuntu-latest`:

| Run | Result |
|-----|--------|
| [43103063083](https://github.com/yzhou216/runner-sandbox/actions/runs/43103063083) | Failed: the runner has no git identity for the scratch commits that `merge-tree` merges |
| [43103242741](https://github.com/yzhou216/runner-sandbox/actions/runs/43103242741) | Passed |

## Outcome

The rewritten master is `d55ee76c72122451001f346c878d817f41e1f075`, with
24 commits, and the rewritten c2y is
`9e6210676b51bae4b2e4daf7e1154339ffc854dd`, with 15 more.  A run of
relicense.py on the laptop produced the same commits.  The `rewritten`
artifact carried them to the laptop, which pushed them to rv64_emu.

Every commit built without warnings.  The emulated results of the master
commits match the native ones on all 41 base cases from the commit that
adds `sw` on, and the master tip passes all 287 cases with every cache
configuration.  Every c2y commit from the one that adds the tests on
passes all of its cases, 371 at first and 404 at the end.
