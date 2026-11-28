<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# flake-check: cleanup of the graphical and text-only NixOS profiles

This branch records the flake-check runs for a cleanup of
`systems/nixos/graphic.nix`, `systems/nixos/t3code-themes.nix` with
`systems/nixos/t3code-themes.el`, and `systems/nixos/tty.nix` in
[yzhou216/flake](https://github.com/yzhou216/flake), on top of master at
1dd2043.  The harness itself is described on the
`flake/2026-11-28-refactor-harness` branch; `flake-check/flake.patch` holds
the series under test.

## Series

| Commit | Change | Derivations |
| --- | --- | --- |
| 801472d | `tty: remove the unused profile` | All the same |

`systems/nixos/tty.nix` has never been imported by any configuration in the
history of the flake; router imports `commons.nix` directly.  The graphical
profile and the t3code theme module were reviewed as well, and left
unchanged, since nothing in them was dead, deprecated, or redundant without
also changing a derivation.

## Runs

| Run | Tested | Outcome |
| --- | --- | --- |
| 46136242485 | 801472d, `[no-darwin]` | On ubuntu-latest every GNU/Linux target is the same as the base (galago and router toplevel and diskoScript, bcachefs-iso toplevel and ISO image); `packages.x86_64-linux.openwrt` fails in both with the known hash mismatch of the OpenWrt package index.  No evaluation warning or error appears on one side only.  nixfmt, deadnix, and statix counts are unchanged at 1, 14, and 36 lines.  The macOS job was skipped, as no Darwin configuration imports the removed file |

## Baseline

The flake's master at 1dd2043 with no patch, run 46133564425: all GNU/Linux
targets evaluate except `packages.x86_64-linux.openwrt`; on macOS
`darwinConfigurations.lexi` evaluates and `darwinConfigurations.iris` fails.
