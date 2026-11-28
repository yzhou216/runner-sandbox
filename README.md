<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# flake-check: derivations of the flake before and after a change

A harness for refactoring [yzhou216/flake](https://github.com/yzhou216/flake)
without building it.  `.github/workflows/flake-check.yml` clones the flake
from <https://gitlab.com/yiyuz/flake.git> at the commit in
`flake-check/base`, applies `flake-check/flake.patch` to a second copy, and
evaluates every attribute in `flake-check/targets` in both, at the same time
so that the impure fetches of the flake resolve alike.  The results list, for
each target, whether its derivation is the same, changed, or fails to
evaluate, then the evaluation warnings and errors that only one side prints,
the nixfmt, deadnix, and statix findings, and `nix-diff` of each changed
derivation.

- The Darwin configurations are evaluated on `macos-latest`, the rest on
  `ubuntu-latest`.
- A commit message with `[no-darwin]` skips the macOS job, one with `[pure]`
  evaluates the candidate without `--impure`, and one with `[build]` builds
  what `flake-check/build` lists, after `nothing-but-nix` frees disk space.
- If the patch adds flake inputs, the job runs `nix flake lock` on the
  candidate and saves the new `flake.lock` as `flake.lock.new` in its
  artifact.

## Use

Branch off this branch, commit changes in a clone of the flake on top of the
base commit, and from a clone of this repository run:

```sh
flake-check/submit.sh <flake clone> "flake-check: <what the run tests>"
flake-check/wait.sh <run id>    # again until it prints the results
```

To check every commit of a series on its own, run
`flake-check/submit-series.sh <flake clone> "<message>"` instead: it writes
one patch per commit to `flake-check/series`, and
`.github/workflows/flake-series.yml` evaluates each commit against the
commits before it, one job per commit, on macOS too for commits that touch
what the Darwin configurations read; `wait.sh` then prints a table of what
each commit changes.

`submit.sh` writes the diff of the clone against the base to
`flake-check/flake.patch`, commits and pushes it, and prints the run ID;
`wait.sh` waits up to eight minutes per call, then prints `results.md` of
each job and extracts the artifacts.

## Baseline

The flake's master at 1dd2043, with no patch:

| Run | Outcome |
| --- | --- |
| 46133564425 | All GNU/Linux targets evaluate, in 4 minutes, except `packages.x86_64-linux.openwrt`, which stops at a hash mismatch in the package index of the OpenWrt image builder; on macOS, in 3 minutes, `darwinConfigurations.lexi` evaluates and `darwinConfigurations.iris` fails.  nixfmt finds `flake.nix` unformatted; deadnix prints 14 lines and statix 36 |
| 46134021458 | Same, after the macOS job became optional |
| 46134384028 | `[pure]` check of the base itself |
| 46164547092 | Series mode on three commits pinning Emacs and emacs-overlay: every commit leaves every derivation the same on GNU/Linux and macOS |
