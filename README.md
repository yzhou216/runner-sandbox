<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# flake-check: structure of systems/nixos/commons.nix

This branch tested a series of commits on
[yzhou216/flake](https://github.com/yzhou216/flake) at 1dd2043 that splits
`systems/nixos/commons.nix`, the NixOS module that galago and router share,
into focused modules and drops settings that are NixOS defaults or have no
effect.  The harness is the one of `flake/2026-11-28-refactor-harness`:
`.github/workflows/flake-check.yml` evaluates every attribute in
`flake-check/targets` at the base and with `flake-check/flake.patch` applied,
the Darwin configurations on `macos-latest` and the rest on `ubuntu-latest`.

## Series

1. `commons: ignore the argument of the locale function`
2. `commons: merge the definitions of the t3code environment`
3. `commons: move the t3code service into t3code.nix`
4. `commons: move the emacs overlays into emacs.nix`
5. `commons: drop commented-out settings of the configuration.nix template`
6. `commons: drop settings that nixos sets by default`
7. `commons: drop the networkmanager settings, which nothing enables`
8. `commons: move networking, openssh, and tailscale into network.nix`

## Runs

| Run | Tested | Outcome |
| --- | --- | --- |
| 46136437149 | The whole series, before three commit messages were reworded | Every derivation the same as the base on GNU/Linux and macOS; `packages.x86_64-linux.openwrt` and `darwinConfigurations.iris` fail in both, as at the base.  deadnix 14 lines to 10, statix 36 to 21, nixfmt unchanged |
| 46136900370 | Commits 1 to 4 only, the moves before any setting is dropped, `[no-darwin]` | Every GNU/Linux derivation the same as the base |
| 46136949098 | The final series, commits 1 to 8 | Every derivation the same as the base on GNU/Linux and macOS, with the same errors, warnings, and lint counts as 46136437149 |

That the galago and router toplevels stay the same without
`networking.networkmanager.wifi.backend` and the `networkmanager` group
confirms that NetworkManager is enabled on neither system.

## Baseline

The flake's master at 1dd2043, with no patch, is run 46133564425: all
GNU/Linux targets evaluate except `packages.x86_64-linux.openwrt`, which stops
at a hash mismatch in the package index of the OpenWrt image builder; on
macOS `darwinConfigurations.lexi` evaluates and `darwinConfigurations.iris`
fails.  nixfmt finds `flake.nix` unformatted; deadnix prints 14 lines and
statix 36.
