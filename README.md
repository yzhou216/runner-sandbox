<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# flake-check: the modules that NixOS and Darwin share

This branch tests a refactor of the shared modules under `systems/` of
[yzhou216/flake](https://github.com/yzhou216/flake) (`nix.nix`,
`nixpkgs.nix`, `shell.nix`, `dev.nix`, `editors.nix`, `vc.nix`, `man.nix`)
and of how `systems/nixos/commons.nix` and `systems/darwin/default.nix`
import them.  It branches off `flake/2026-11-28-refactor-harness`; the
harness is described there.  Every run compares the flake's master at
1dd2043 with the series in `flake-check/flake.patch`, on `ubuntu-latest`
and, for the Darwin configurations, on `macos-latest`.

## Series

1. `systems: import the shared modules through one module`: a new
   `systems/commons.nix` imports the seven shared modules, and both
   platform modules import it instead of listing the seven each.
2. `shell: enable tmux for nixos and darwin alike`: `programs.tmux.enable`
   moves from both platform modules into `systems/shell.nix`.
3. `shell: check for darwin through stdenv.hostPlatform`: drops the
   deprecated `stdenv.isDarwin`.
4. `man: set the nixos-only developer documentation in commons`:
   `documentation.dev.enable`, which nix-darwin lacks, moves from the
   platform check in `systems/man.nix` to `systems/nixos/commons.nix`.
5. `darwin: drop nix-init, which dev.nix installs`: removes a repeated
   package.

## Harness change

`flake-check/run.sh` ran `nix-diff` only in the GNU/Linux job.  On this
branch it also runs in the macOS job, so that a changed Darwin
configuration comes with its `nix-diff`.

## Runs

| Run | Tested | Outcome |
| --- | --- | --- |
| 46136196704 | Series 1 to 5 | Every GNU/Linux target the same; `darwinConfigurations.lexi.system` changed; the Darwin warning about `stdenv.isDarwin` gone.  No `nix-diff` on macOS yet |
| 46136658012 | Series 1 to 4 | Every target the same on both platforms, `lexi` included; the `stdenv.isDarwin` warning gone.  So the first four commits leave every derivation as it was |
| 46136681397 | Series 1 to 5, with `nix-diff` on macOS | `lexi` changed: in the `system-path` and `system-applications` environments, the only difference is one fewer `nix-init-0.3.7` entry in `chosenOutputs` (86 characters of the JSON); `etc` and `darwin-system` change only through them |
| 46137143959 | Final series 1 to 5 | Same as 46136681397: GNU/Linux all the same, `lexi` changed to `a64ir8dbwz8s` by the removed `nix-init` entry only, `iris` fails in both as on master |

In every run, `packages.x86_64-linux.openwrt` failed in both trees on the
hash mismatch of the OpenWrt package index, as on master, and nixfmt (1
line, `flake.nix`), deadnix (14 lines), and statix (36 lines) reported the
same counts as on master.
