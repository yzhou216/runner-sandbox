<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# rv64_emu: length check of to_upper

This session checked every commit of the c2y branch of
[yzhou216/rv64_emu](https://github.com/yzhou216/rv64_emu) after the length
check of `to_upper` moved from "rv64_emu: check more arguments and tidy up
the harness" into "build: compile as c2y with warnings".  With the `-O2` of
the fortify hardening of Nixpkgs, `-Wall` reported the bounded `strncpy()`
calls that the check replaces, so the Nix builds of the first twelve
commits had three warnings each.  The rewritten commits travelled as
[rebuild/c2y-rebase.bundle](rebuild/c2y-rebase.bundle), on top of master
`b098cbc797116400b3f894a12d42711b6db7fc89`, and
[rebuild/check-c2y.sh](rebuild/check-c2y.sh) ran `nix flake check` and
`make check` under `qemu-riscv64` on each.

## Runs

All on `ubuntu-latest`:

| Run | Result |
|-----|--------|
| [43122039412](https://github.com/yzhou216/runner-sandbox/actions/runs/43122039412) | Passed |

## Outcome

All 15 commits passed both checks, and neither the Nix builds nor the make
builds gave a warning.  The golden-output tests passed from the commit
that adds them on, 371 cases at first and 404 at the end.  The rewritten
c2y, `b3abf1faba015e71fb1588c4d41d7b659e614bbf`, was pushed to rv64_emu
from the laptop.
