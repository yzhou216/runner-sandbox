<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# flake-check: correctness review of the flake

This branch checks a series of bug fixes to
[yzhou216/flake](https://github.com/yzhou216/flake) on top of master at
1dd2043, with the harness of `flake-check/` (see `run.sh`, `build.sh`, and
`.github/workflows/flake-check.yml`).  `flake-check/flake.patch` holds the
final series:

1. `iris: accept the module arguments of nix-darwin`
2. `commons: drop the settings of the disabled networkmanager`
3. `lexi: explain why the nixbld group id is set`
4. `scripts: switch to the iris configuration in darwin_setup.sh`
5. `scripts: match only the sip status line in darwin_setup.sh`
6. `scripts: pass the password to sudo verbatim in darwin_setup.sh`
7. `scripts: run darwin-rebuild as root in darwin_setup.sh`
8. `t3code: export the themes with the modus themes of emacs 30`

## Probes

Besides the usual targets, `flake-check/targets` evaluates the units of
NetworkManager, dhcpcd, and iwd on galago: a unit evaluates only if its
service is enabled.  `flake-check/build` builds the unit of t3code on
router, which depends on the Modus themes that t3code-themes.el exports
with the Emacs of router.  Both files quote the unit names in single
quotes, which keep the double quotes of the attribute path through `xargs`
in `run.sh` and `build.sh`.

## Runs

| Run | Tested | Outcome |
| --- | --- | --- |
| 46136571793 | Commits 1 to 7, with the probes | Cancelled: the probes lacked the single quotes, so `xargs` dropped their double quotes |
| 46136616319 | Commits 1 to 7, with the probes | `darwinConfigurations.iris` evaluates (it fails in the base); every other target is the same in base and candidate.  The NetworkManager unit is missing on galago in both, while the dhcpcd and iwd units exist and are the same |
| 46137400413 | `[build] [no-darwin]`: commits 1 to 7, which leave the unit of t3code as in the base | The unit of t3code on router fails to build: `t3code-modus-themes` stops with "Symbol's function definition is void: modus-themes-retrieve-palette-value", since Emacs 30 bundles Modus themes 4.4 |
| 46160206696 | `[build]`: the final series, commits 1 to 8 | The unit of t3code on router builds.  `nixosConfigurations.galago` and `router` change, and nix-diff traces both changes to the source `t3code-themes.el` alone; every other GNU/Linux target is the same.  On macOS, iris evaluates and lexi is the same.  nixfmt, deadnix, and statix print 1, 14, and 36 lines, as in the base |

`packages.x86_64-linux.openwrt` fails in base and candidate alike, at a
hash mismatch in the package index of the OpenWrt image builder, as in the
baseline run 46133564425.

## Local check of the theme export

The theme export runs in a build, so it was also run outside Nix with
`emacs -Q --batch -l t3code-themes.el <dir>`:

- With Emacs 32.0.50 and its Modus themes 5.3.0, the script before and
  after commit 8 writes identical files.
- With the Modus themes 4.4.0 files of Emacs 30 first on
  `custom-theme-load-path`, the script before commit 8 fails as on router,
  and the script after it resolves all 61 color roles of each variant.
