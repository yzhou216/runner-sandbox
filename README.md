<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# flake-check: machine definitions and disk layouts

This branch evaluated a series of commits to
[yzhou216/flake](https://github.com/yzhou216/flake) on top of master at
1dd2043, which covers the machine definitions and the disko layout: the
notes on testing router in a VM, the disko image builder workaround for the
out-of-tree bcachefs module, and the OpenWrt image, which failed to
evaluate on master.  The harness itself is described in the README of the
`flake/2026-11-28-refactor-harness` branch: `flake-check/submit.sh` sends
the diff of a clone of the flake against the base, and
`flake-check/wait.sh` prints which derivations are the same, changed, or
fail to evaluate.

## Series

1. `router: correct the notes on testing in a VM`: comment in `flake.nix`
   only.
2. `disko: move the bcachefs image builder workaround into the layout`:
   moves `disko.imageBuilder` from `machines/nixos/router.nix` into
   `disko/bcachefs-image-builder.nix`, which `disko/bcachefs.nix` imports,
   so that galago gets it too.
3. `openwrt: explain the hash mismatches of the package feeds`: comment in
   `flake.nix` only.
4. `flake: update openwrt-imagebuilder for fresh feed hashes`: moves the
   locked revision from 38bd2b4 (26 November) to 8cae777 (27 November),
   whose NAR hash was computed from the GitHub tarball and checked by
   computing that of 38bd2b4 the same way.

## Targets

To the default targets, this branch added `vmWithDisko` and `diskoImages`
of galago and router, the outputs that the image builder settings affect.

## Runs

| Run | Tested | Outcome |
| --- | --- | --- |
| 46136443465 | Commits 1, 3, and 4 | Every derivation the same on GNU/Linux and macOS, router's `vmWithDisko` included; `packages.x86_64-linux.openwrt` evaluates, where the base stops at a hash mismatch of `mips_24kc-base-packages.adb`.  Lint counts unchanged |
| 46136962216 | Commits 1 to 4 | Same as above, and router's `vmWithDisko` and `diskoImages` stay the same.  Galago's `vmWithDisko` and `diskoImages` change as intended: the kernel modules of the image builder become `linux-6.18.65` with `kernel-modules-with-bcachefs`, like router's, instead of `linux-7.4-rc4` alone |
| 46137411623 | Final series, the four commits above | Same as 46136962216.  Galago's system changed in both base and candidate between runs, from the unpinned Emacs fetches, but stays the same between them.  `darwinConfigurations.lexi` the same, `darwinConfigurations.iris` failing in both, as on master.  nixfmt 1, deadnix 14, and statix 36 lines in both |

## OpenWrt

The hashes in nix-openwrt-imagebuilder of the mips_24kc feeds of 25.12.6
(base, luci, packages, routing, telephony) and of their `sha256sums` all
changed between 38bd2b4 and 8cae777, while those of the lantiq/xrx200
target did not.  Fetching every index that the FRITZ!Box 7412 image uses
from downloads.openwrt.org on 28 November matched 8cae777 in full.  The
history of the hash files shows those of 24.10.8 and 25.12.6 changing every
day or two, and those of 22.03.7 and 23.05.6 unchanged since January.
Updating the input fixes the evaluation until the feeds move again; pinning
the input does not help, and neither does another profile of a maintained
release.
