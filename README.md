<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# flake-check: CI and checks of the flake

This branch tests a series for [yzhou216/flake](https://github.com/yzhou216/flake)
that replaces its only workflow, `build-galago.yml`, which had failed on
every run since October 31, with CI that checks the flake as committed:

- `flake.nix` gains `formatter` (nixfmt-tree, for `nix fmt`) and `checks`:
  `checks.x86_64-linux.nixfmt` runs `nixfmt --check` on every Nix file, and
  `checks.aarch64-darwin.<host>` is the system of each nix-darwin
  configuration, which `nix flake check --all-systems` evaluates on
  GNU/Linux.
- A new `check.yml` runs `nix flake check --impure --all-systems
  --keep-going` on each push and pull request, after refreshing only the
  `openwrt-imagebuilder` input.
- `build-galago.yml` builds `flake.lock` as committed on each push to
  master, and runs `nix flake update` only in a daily scheduled run, which
  keeps the updated lock as an artifact.  Runs of the same kind cancel each
  other, the build step times out before the job, Nix accepts the
  substituters of the flake's `nixConfig`, it installs upstream Nix like
  `check.yml`, and every action is pinned to a commit.

The final series, from `flake-check/flake.patch`:

```
2288c78 flake: format with nix fmt
1febbe9 flake: check the formatting of the nix files
ed9148d flake: check the darwin configurations
d43d33d workflows: run nix flake check on each push
c967361 workflows: refresh the openwrt hashes before the check
bad6bef workflows: build galago from flake.lock, and update it once a day
aeb5810 workflows: keep long galago builds from overlapping and being cut off
a45d80b workflows: substitute from the caches of flake.nix in the galago build
25b5a96 workflows: install upstream nix in the galago build
90548b4 workflows: pin the actions of the galago build to commits
```

## How it was tested

- `.github/workflows/flake-check.yml`, the harness of the base branch,
  compares the derivations of base and candidate.  `flake-check/targets`
  adds the new outputs: `formatter.x86_64-linux`,
  `formatter.aarch64-darwin`, `checks.x86_64-linux.nixfmt`, and
  `checks.aarch64-darwin.{iris,lexi}`, evaluated on GNU/Linux.
- `.github/workflows/flake-ci.yml` runs copies of the two workflows of the
  series on the base commit with the patch applied;
  `flake-ci/checkout.sh` stands in for `actions/checkout`, and comments
  mark each other difference.  Job `check` is `check.yml` as it is.  Job
  `check-fixed` first runs `flake-ci/fixups.sh`, which runs `nix fmt`
  and fixes `machines/darwin/iris.nix`, to show the check once the
  problems that it finds on the base are gone.  Job `build` runs
  `build-galago.yml` both as a push and as the scheduled update, with
  `nix build --dry-run` instead of the build of galago, which takes hours,
  and a small build that records its build directory.
- `.github/workflows/build-dir-probe.yml` finds where builds run after
  nothing-but-nix with `build-dir = /nix/build`, with either installer.

## Runs

| Run | Outcome |
| --- | --- |
| 46137170674 | Harness on the first version of the series: every existing target `same` on GNU/Linux and macOS.  `checks.aarch64-darwin.lexi`, evaluated on GNU/Linux, gives `iskkgcvkzcy9`, the derivation that `darwinConfigurations.lexi.system` gives on macOS; `checks.aarch64-darwin.iris` fails like `darwinConfigurations.iris.system`.  deadnix and statix counts unchanged |
| 46137170777 | flake-ci on the same: `check` fails on exactly three problems of the base: `flake.nix` not formatted, `iris.nix` "called with unexpected argument 'lib'", and a hash mismatch of the OpenWrt package index.  `check-fixed`, which then also updated `openwrt-imagebuilder`, passes in 2 minutes at 10:52 UTC.  `nix fmt` formatted 21 files, 1 changed.  The build's update step, summary, and flake.lock artifact work |
| 46137810220 | flake-ci with the build as push and as update: galago needs 73 and 70 derivations built, and 3575 and 3593 paths fetched.  The listing of `/nix/build` in this run proves nothing, since `sleep` is missing in the build sandbox |
| 46160287616, 46160466233 | build-dir probe: the first is void for the same reason; the second shows that builds run in `/nix/build/nix-*` with both cachix/install-nix-action (Nix 2.36.0) and DeterminateSystems/nix-installer-action (Determinate Nix 3.24.3) |
| 46160613893, 46160613830 | Series with the OpenWrt refresh in `check.yml`: harness all `same`; `check-fixed` now fails only on the OpenWrt index at 15:35 UTC, with the Image Builder of November 27 that passed at 10:52 UTC, so the refresh narrows but does not close that gap |
| 46161308313 | Harness on the final series: every existing target `same` on GNU/Linux and macOS, lints unchanged (nixfmt 1, deadnix 14, statix 36 lines), and the new outputs evaluate, except iris |
| 46161308622 | flake-ci on the final series: `check` fails on the three problems of the base; `check-fixed` fails only on the OpenWrt index; `build` succeeds as push (40 derivations to build, 3573 paths to fetch) and as update (45 to build), and builds run in `/nix/build` |

## Why build-galago.yml failed

Read from `gh run list -R yzhou216/flake` and the failed logs: 93
successful runs, the last on October 31, then failures on every run, each
on a change in the latest inputs that the hourly `nix flake update`
brought in: nixpkgs refused to evaluate gtk+ 2 (November 1 and 2), dropped
brave-origin (November 4 to 8), the bcachefs out-of-tree module did not
build against Linux 7.4-rc1 to rc3 of `linuxPackages_testing` (November 10
to 23), emacs-git of the unpinned overlay failed on a patch that upstream
had applied, and from November 23 the kernel failed on the xhci patch of
`commons.nix`, which Linux 7.4-rc4 contains and which master has since
dropped.  The new lock was thrown away, so no run ever built what is
committed, and some runs hit the limit of six hours building Electron and
the kernel.
