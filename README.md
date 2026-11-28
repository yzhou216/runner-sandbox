<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# flake-check: refactor of the outputs of flake.nix

This branch tested a refactor of the outputs section of
[yzhou216/flake](https://github.com/yzhou216/flake) against its master at
1dd2043, with the flake-check harness that this branch inherits from
`flake/2026-11-28-refactor-harness`: `flake-check/flake.patch` holds the
series under test, and the workflow evaluates every attribute in
`flake-check/targets` before and after it, on `ubuntu-latest` and, for the
Darwin configurations, on `macos-latest`.

## Series

1. `flake: build galago and router with a shared mkNixos helper`
2. `flake: build iris and lexi with a shared mkDarwin helper`, which takes
   the user of nix-homebrew from `system.primaryUser`
3. `openwrt: move the fritzbox 7412 configuration into machines/openwrt`

## Targets

This branch adds `packages.x86_64-linux.openwrt.config.files` to
`flake-check/targets`.  The OpenWrt image itself fails to evaluate in both
trees, at a hash mismatch in the package index of the image builder, so its
derivation cannot be compared; the derivation of the files that the moved
configuration puts in the image, reached through the `config` that the image
builder passes through, evaluates without that index and can be.

## Runs

| Run | Tests | Outcome |
| --- | --- | --- |
| 46136057092 | The whole series | On GNU/Linux, every derivation is the same as in the base, `packages.x86_64-linux.openwrt.config.files` included, and `packages.x86_64-linux.openwrt` fails in both with the same hash mismatch.  On macOS, `darwinConfigurations.lexi` is the same and `darwinConfigurations.iris` fails in both with the same error.  The error and warning lines of every target match between base and candidate.  nixfmt still finds only `flake.nix` unformatted, at the long `fetchPrCommit` lines that the series leaves alone; deadnix prints 14 lines and statix 36, as in the base |

The baseline, the base with no patch, is run 46133564425 on
`flake/2026-11-28-refactor-harness`.
