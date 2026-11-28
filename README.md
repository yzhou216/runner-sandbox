<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# flake-check: the nixpkgs patching of yzhou216/flake

This branch checks a refactor of the patching machinery at the top of
`flake.nix` in [yzhou216/flake](https://github.com/yzhou216/flake):
`nixpkgs-patched`, which applies commits of open nixpkgs pull requests
with `applyPatches`, and `nixosSystem-patched`, which evaluates NixOS from
the patched tree.  The harness is the one of
`flake/2026-11-28-refactor-harness`: `flake-check/submit.sh` and
`flake-check/wait.sh`, with the `.github/workflows/flake-check.yml`
workflow, evaluate every attribute in `flake-check/targets` before and
after `flake-check/flake.patch`, on `ubuntu-latest` and, for the Darwin
configurations, on `macos-latest`.

## Series under test

Two commits on master at 1dd2043, with every derivation unchanged:

1. `flake: move the nixpkgs patching into patched.nix` moves
   `nixpkgs-patched` and `nixosSystem-patched` into a new `patched.nix`,
   which `flake.nix` imports with
   `inherit (import ./patched.nix inputs) nixosSystem-patched;`.  It also
   passes the file of the caller's `modules` as `modulesLocation`, so that
   errors in inline modules keep naming `flake.nix`.
2. `patched: list the commits of each pull request as data` drops the
   unused pull request argument of `fetchPrCommit`, which deadnix
   reported, and lists each pull request with its URL and its commits.
   `applyPullRequests` takes a name, a source tree, and that list, so that
   other inputs, such as disko, can be patched the same way.

## Runs

| Run | What | Outcome |
| --- | --- | --- |
| 46136286720 | Commit 1 alone, `[no-darwin]` | Every GNU/Linux derivation the same; `packages.x86_64-linux.openwrt` fails in both, at a hash mismatch of a different OpenWrt index each time.  nixfmt clean (base: `flake.nix` unformatted); deadnix 16 lines, since the unused `pr` moves into a file of its own |
| 46136703946 | Commits 1 and 2, `[no-darwin]` | Every GNU/Linux derivation the same; nixfmt clean, deadnix 12 lines (base 14), statix 36 lines (base 36) |
| 46136746638 | `hotfix/galago-bcachefs-encryption` (b5d6424) rebased onto the series, with its disko patches moved into `patched.nix` as a `disko-patched` call of `applyPullRequests`, against b5d6424 itself as the base, `[no-darwin]` | Every GNU/Linux derivation the same as on the hotfix branch, including both `diskoScript` targets, which use the patched disko |
| 46136784360 | Experiment, not in the series: `nixosSystem-patched` with the lib extended by `lib/flake-version-info.nix` of nixpkgs, as `nixpkgs.lib.nixosSystem` has it, `[no-darwin]` | galago and router `toplevel` change, as expected: their version goes from `27.05pre-git` to `27.05.20261127.7387b85`, in the system name, `nixos-version` (which gains `nixpkgsRevision`), `/etc/os-release`, and `/etc/issue` |
| 46137222658 | Final: commits 1 and 2, on both platforms | Every derivation the same on GNU/Linux and macOS (`darwinConfigurations.lexi` the same, `darwinConfigurations.iris` failing in both); nixfmt clean, deadnix 12 lines, statix 36 lines |

For run 46136746638, `flake-check/base` pointed at b5d6424 for that one
commit; it is back at 1dd2043 for the runs after it.
