<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# flake-check: formatting and linters

This branch tests a series for
[yzhou216/flake](https://github.com/yzhou216/flake), on top of master at
1dd2043, that clears the findings of nixfmt and deadnix, applies the statix
suggestion that improves the code, and adds a formatter output so that
`nix fmt` works.  The harness itself is unchanged; see the branch
`flake/2026-11-28-refactor-harness` for how it works.  This branch adds
`formatter.x86_64-linux` and `formatter.aarch64-darwin` to
`flake-check/targets`, both evaluated on GNU/Linux.

## Series

1. `treewide: remove unused bindings` drops the five bindings that deadnix
   reports: the `homebrew-core`, `homebrew-cask`, and `homebrew-bundle`
   arguments of `outputs`, which the taps read through `inputs`, the
   ignored pull request number of `fetchPrCommit`, and the `locale`
   argument in `i18n.extraLocaleSettings`.  With two arguments, nixfmt
   keeps each call of `fetchPrCommit` on one line, so this also leaves
   `flake.nix` formatted.  A separate nixfmt commit before it would have
   broken those six calls over three lines each, only for this commit to
   join them again, so there is none.
2. `commons: merge the repeated systemd attributes` fixes the statix
   `repeated_keys` warning: `systemd` was defined three times at the top
   level of `systems/nixos/commons.nix`.
3. `flake: add nixfmt-tree as the formatter` adds `formatter` for
   x86_64-linux and aarch64-darwin.  nixfmt-tree runs nixfmt through
   treefmt, which formats the whole tree when `nix fmt` passes no paths.

The three statix `empty_pattern` warnings, for `{ ... }:` in
`machines/nixos/state-version.nix`, `machines/darwin/state-version.nix`,
and `systems/nixpkgs.nix`, are left alone.  statix exempts NixOS modules
from this lint, but recognizes a module only by a top-level `imports`;
these three files are modules without one.  The repository writes
argument-less modules as `{ ... }:`, as in `machines/nixos/galago.nix`, and
`_:` would hide that the files are modules.

## Runs

| Run | Tests | Outcome |
| --- | --- | --- |
| 46136127423 | The final series, a00a749, cc3c971, and c7c52da | Every GNU/Linux and macOS target has the same derivation as the base, or fails in both (`packages.x86_64-linux.openwrt` and `darwinConfigurations.iris.system`, as in the base).  `formatter.x86_64-linux` and `formatter.aarch64-darwin` evaluate in the candidate and are missing in the base.  No evaluation warnings or errors are new.  nixfmt goes from 1 line to 0, deadnix from 14 to 0, and statix from 36 to 21, the three `empty_pattern` warnings |

The baseline is run 46133564425 of master with no patch.
