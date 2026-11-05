<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# scores: flake-parts platform tests

Tested the `flake.nix` migration from `flake-utils` to `flake-parts` from
[scores](https://github.com/yzhou216/scores), using `nixpkgs.lib.platforms.all`
and an explicitly imported Nixpkgs for every system.

## Final run

Run [41940945009](https://github.com/yzhou216/runner-sandbox/actions/runs/41940945009)
ran `nix flake check` and `nix build` on the source snapshot in `src-tree/`.

- `ubuntu-24.04` (x86_64 Linux): passed.
- `ubuntu-24.04-arm` (aarch64 Linux): passed.
- `macos-15` (aarch64 Darwin): passed.
- FreeBSD 15.1: Nix 2.32.4 installed and evaluated the flake, but Nixpkgs
  rejected the Linux-only `iproute2` package in LilyPond's dependency closure.
- NetBSD 11.1 and 10.2: no `nix` binary package was available from `pkg_add`.
- OpenBSD 8.0: no `nix` binary package was available.  OpenBSD 7.8 provided
  Nix 2.3.16, which predates the `nix flake` command.

The BSD results are Nix or Nixpkgs platform limitations, not failures in the
`flake-parts` migration.  Current Nixpkgs master has also dropped
`x86_64-darwin`, so that host was excluded from the final native matrix.

Earlier runs [41939928668](https://github.com/yzhou216/runner-sandbox/actions/runs/41939928668)
and [41940508957](https://github.com/yzhou216/runner-sandbox/actions/runs/41940508957)
record the initial setup fixes: importing Nixpkgs directly for non-flake-exposed
systems, leaving the generated lock file untracked as in the source project,
and installing Git for FreeBSD flake inputs.
