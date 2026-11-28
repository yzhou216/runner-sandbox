<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# flake-check: the nix-darwin side of the flake

This branch tested a series of commits on the nix-darwin side of
[yzhou216/flake](https://github.com/yzhou216/flake), on top of master at
1dd2043, with the harness of `flake/2026-11-28-refactor-harness`: each run
evaluates the targets in `flake-check/targets` in the base and with
`flake-check/flake.patch` applied, the Darwin configurations on
`macos-latest` and the rest on `ubuntu-latest`.  The patch in
`flake-check/flake.patch` is the final series.

## Series

1. `darwin: accept the module arguments in iris`: `machines/darwin/iris.nix`
   started with `{ }:`, which the module system cannot call, so
   `darwinConfigurations.iris` failed to evaluate.
2. `shell: test the host platform for darwin`: replaces the deprecated
   `stdenv.isDarwin` in `systems/shell.nix`.
3. `flake: define the darwin systems with one function`: iris and lexi share
   one local function, and nix-homebrew takes its user from
   `system.primaryUser`.
4. `darwin_setup: read and pass the password verbatim`: shellcheck quoting.
5. `darwin_setup: switch to the iris configuration as root`: the script
   switched to `.#m1`, which no longer exists, and without root, which
   nix-darwin requires.
6. `shell: enable tmux once for nixos and darwin`: one definition in
   `systems/shell.nix` instead of one per platform module.
7. `flake: stop binding the homebrew taps in the outputs`: unused arguments
   that deadnix reports.
8. `darwin: take no module arguments in the state version`: the empty
   pattern that statix reports.
9. `flake: fetch nix-darwin and nix-homebrew from their current owners`:
   new owners in `flake.nix` and `flake.lock`, at the same revisions and NAR
   hashes.

## Runs

| Run | Tested | Outcome |
| --- | --- | --- |
| 46136516691 | The series above, before a commit subject was reworded | `darwinConfigurations.iris` evaluates (base error), `darwinConfigurations.lexi` and every GNU/Linux target are the same as in the base, and `packages.x86_64-linux.openwrt` fails in both, as in the baseline.  The `stdenv.isDarwin` warning is gone; the `'vam'` warning remains, from nix-darwin's `programs.vim` module.  `nix flake lock` left `flake.lock` unchanged.  deadnix fell from 14 lines to 8 and statix from 36 to 29; nixfmt still reports only `flake.nix`, for lines outside the series |
| 46136930960 | The final series, the same tree as above | Same as 46136516691 |

The baseline, master with no patch, is run 46133564425 of the harness
branch: iris fails, lexi evaluates, and Darwin warns about `'vam'` and
`stdenv.isDarwin`.
