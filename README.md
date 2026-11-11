<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# rv64_emu: c2y rebase

This session checked every commit of the c2y branch of
[yzhou216/rv64_emu](https://github.com/yzhou216/rv64_emu) after its
rebase onto master, which had gained a flake, CI, and REUSE compliance.
The rebased commits travelled as
[rebuild/c2y-rebase.bundle](rebuild/c2y-rebase.bundle), on top of master
`b098cbc797116400b3f894a12d42711b6db7fc89`, since they were not public
yet.  [rebuild/check-c2y.sh](rebuild/check-c2y.sh) ran `nix flake check`
on each commit, and `make check` with the RISC-V cross toolchain of the
nixpkgs that the flake locks (GCC 16.2.0) under `qemu-riscv64`.

## Runs

All on `ubuntu-latest`:

| Run | Result |
|-----|--------|
| [43115697604](https://github.com/yzhou216/runner-sandbox/actions/runs/43115697604) | Passed |

## Outcome

All 15 commits passed both checks.  `nix flake check` passed the REUSE
lint, the formatting check, and the comparison of each test program with
its native assembly on every commit, and the golden-output tests from the
commit that adds them on, 371 cases at first and 404 at the end, as did
`make check`.  The make builds gave no warnings.  The Nix builds of the
first twelve commits gave three `-Wstringop-truncation` warnings, which
the `-O2` of the fortify hardening of Nixpkgs brings out in the
`strncpy()` calls of `to_upper_test()`, until the commit that checks more
arguments replaces them.

The rebased c2y, `4d84d65a338fd62b462052a41b50ce1016887a89`, was pushed to
rv64_emu from the laptop.
