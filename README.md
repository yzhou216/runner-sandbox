<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# flake-check: pure evaluation of the flake

This branch tested a series for
[yzhou216/flake](https://github.com/yzhou216/flake) on master at 1dd2043 that
lets the flake evaluate without `--impure`.  The base fetched emacs-overlay
with `builtins.fetchTarball` and no `sha256`, and Emacs master with
`builtins.fetchGit` and only a `ref`, both in `systems/nixos/commons.nix`.
The series turns both into flake inputs that the module reaches through the
`inputs` special argument:

1. `commons: pin emacs-overlay as a flake input`
2. `commons: pin emacs master as a flake input`, with `github:emacsmirror/emacs`
   as a non-flake input
3. `flake: drop --impure from the build commands`, in the comment on router
   and in `.github/workflows/build-galago.yml`

No other module, including those of the Darwin configurations, fetches
anything impurely.  The harness at the base of this branch (see
`flake-check/run.sh` and `.github/workflows/flake-check.yml`) evaluates the
base with `--impure` and, with `[pure]`, the candidate without it.

## Changes to the harness

- `flake-check/targets` adds
  `nixosConfigurations.router.config.system.build.vmWithDisko`, the command
  that the comment on router documents, and
  `nixosConfigurations.galago.pkgs.emacs-git`, whose `src` is the Emacs
  input.
- `flake-check/run.sh` expands the empty `impure` array of a `[pure]` run as
  `${impure[@]+"${impure[@]}"}`.  Bash 3.2 of macOS called the plain
  expansion unbound under `set -u`, so the macOS job of every `[pure]` run
  failed before it evaluated anything.

## Runs

| Run | Tests | Outcome |
| --- | --- | --- |
| 46135993280 | The three commits without `flake.lock`, `[pure]` | The harness locked emacs-overlay at 0a34f1e and Emacs at eb8c188; every GNU/Linux target is the same and evaluates purely, and `packages.x86_64-linux.openwrt` fails in both.  The macOS job stopped at the unbound array above |
| 46136458589 | The series with `flake.lock`, `[pure]`, and the fix to `run.sh` | Every target is the same on GNU/Linux and macOS and evaluates purely; `darwinConfigurations.iris` fails in both |
| 46136825063 | The first commit alone, impure, `[no-darwin]` | Every target is the same |
| 46137237406 | The series after a reworded commit message, `[pure]` | Every target is the same except `nixosConfigurations.galago.pkgs.emacs-git`.  The mirror pushed Emacs master to f55a12f at 10:55Z, while the base was evaluating: its `fetchGit` for galago's toplevel resolved eb8c188, the same as the lock, and its later `fetchGit` for `emacs-git` resolved f55a12f.  nix-diff shows only `src` changed, the impurity that the series removes |
| 46137700214 | The series with the Emacs entry dropped from `flake.lock`, `[pure]`, `[no-darwin]` | The harness relocked Emacs at f55a12f; every target is the same |
| 46160239331 | The final series, with Emacs locked at f55a12f, `[pure]` | Every target is the same on GNU/Linux and macOS and evaluates purely.  `packages.x86_64-linux.openwrt` fails in both on the same `mips_24kc-base-packages.adb-ifd` derivation, each side reporting the hash mismatch of whichever of its fixed-output inputs failed first; `darwinConfigurations.iris` fails in both.  nixfmt, deadnix, and statix print as many lines as on the base: 1, 14, and 36 |

The baseline is run 46133564425 of the harness branch, with 46134384028 as
its `[pure]` check of the base, in which galago and router fail on the
`fetchTarball` without a `sha256`.
