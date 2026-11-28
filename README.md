<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# flake-check: evaluation warnings and deprecations

This branch tested a series on [yzhou216/flake](https://github.com/yzhou216/flake)
that removes the evaluation warnings and deprecations which come from the
flake itself, with the harness of `flake/2026-11-28-refactor-harness`
(see that branch for how `flake-check/submit.sh` and `flake-check/wait.sh`
work).  The base is the flake's master at 1dd2043.

## Series

1. `shell: use stdenv.hostPlatform.isDarwin instead of stdenv.isDarwin`
   (`systems/shell.nix`).  It removes the warning
   `stdenv.isDarwin is deprecated` from the Darwin configurations and
   leaves every derivation the same.
2. `darwin: drop the no_quarantine cask argument`
   (`systems/darwin/default.nix`).  Homebrew removed `--no-quarantine`,
   and the Homebrew that nix-homebrew pins (7.0.15) rejects it, so
   `brew bundle` would fail to install new casks.  Installed casks are
   unaffected, because `brew bundle` does not pass cask arguments when it
   upgrades them.  After review, only the message of this commit was
   reworded; its diff is unchanged.  It changes
   `darwinConfigurations.*.system`: nix-diff shows only the Brewfile losing
   its `cask_args no_quarantine: true` section, and the derivations that
   refer to the Brewfile (`set-environment` through `HOMEBREW_BUNDLE_FILE`,
   then `etc`).

## Harness changes on this branch

- `flake-check/run.sh` runs nix-diff on macOS as well, not only on
  GNU/Linux, so that the change to the Darwin configurations can be
  explained.
- `flake-check/targets` adds `vmWithDisko` of galago and router, which
  `flake.nix` documents as the way to test router in a VM.

## Runs

| Run | Tested | Outcome |
| --- | --- | --- |
| 46136068296 | Both commits, plus a temporary commit (not in the series) that gives `machines/darwin/iris.nix` the arguments `{ ... }` instead of `{ }` | iris evaluates with it, so `{ }` is why iris fails on master.  The only warning left on either Darwin configuration is `'vam' attribute is deprecated`, which comes from the vim module of nix-darwin.  GNU/Linux: all same |
| 46136507794 | Commit 1 alone | Every derivation the same on both platforms; the `stdenv.isDarwin` warning is gone |
| 46136526143 | The series | GNU/Linux: all same.  macOS: lexi changed only in the Brewfile, per nix-diff; iris fails in both, as on master.  The openwrt target failed in both at a different package index of the image builder, which changes upstream |
| 46136924177 | The series, with the `vmWithDisko` targets | Same as the run before; both `vmWithDisko` targets evaluate, with the same derivations and no warnings |
| 46137782866 | The final series after review (commit 2 reworded, same tree as 46136924177) | Same as the run before: GNU/Linux all same, including both `vmWithDisko` targets; lexi changed only in the Brewfile and the derivations that refer to it; iris and openwrt fail in both; the `stdenv.isDarwin` warning appears only in base.  This is the last run, and its `flake.patch` matches the final series exactly |

## Warnings left

- `'vam' attribute is deprecated`, from `programs.vim` of nix-darwin, which
  still passes `vimrcConfig.vam` to `vim-full.customize`
  ([nix-darwin/nix-darwin#832](https://github.com/nix-darwin/nix-darwin/pull/832)
  is open).
- The GNU/Linux configurations print no evaluation warnings, except that
  openwrt fails at import from derivation in both trees before its
  evaluation finishes, so it cannot be checked.
- iris fails to evaluate on master and with the series, because
  `machines/darwin/iris.nix` takes `{ }` instead of `{ ... }`.  Its
  warnings and its changed Brewfile are known only from probe run
  46136068296.
