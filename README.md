<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# galago's disko layout, installed with nixos-anywhere

Tests whether the disk layout of the `galago` machine in
[yzhou216/flake](https://github.com/yzhou216/flake), `disko/bcachefs.nix`
at commit 7f2b444, can be reproduced on an empty disk with nixos-anywhere.
The job runs on ubuntu-latest, under KVM, after `nothing-but-nix` frees disk
space for `/nix`.

`galago-disko/test.sh` runs each phase as a step of its own:

1. It builds galago, its disko script, and the `bcachefs-iso` installer of
   the flake.
2. It boots the installer in QEMU with OVMF and an empty 100G NVMe disk, so
   that the default device of the layout, `/dev/nvme0n1`, applies unchanged.
3. It runs nixos-anywhere with `--store-paths` against the installer over
   SSH, with all of its default phases, up to the reboot.
4. It boots the virtual machine again from the NVMe disk alone, without the
   installer.
5. It runs `galago-disko/check.sh` as root on the installed system.

## Changes for the test

The disko script is the one the flake builds, unchanged; the flake at
7f2b444 and the uncommitted working tree it was tested against yield the
same store path, `f356zmy...-disko`.  Two other parts of the flake needed
changes:

- `galago-disko/galago.nix` replaces `linuxPackages_testing` with
  `linuxPackages_latest`, 7.3.6, and drops the xhci patch.  At 7f2b444,
  galago does not build: the patch no longer applies to 7.4-rc4, which
  carries an equivalent fix upstream, and the bcachefs 1.39.7 module of
  nixpkgs fails to compile against 7.4, whose `xattr_handler.set` takes a
  `const struct mnt_idmap *`.
- `galago-disko/installer.nix` sets `nix.settings.experimental-features`
  and `nix-path` as lists, since the flake sets them as strings, which
  nixpkgs now rejects so that `bcachefs-iso` does not evaluate.  It also
  disables the COSMIC desktop and Firefox, whose dependencies were not yet
  cached, and adds a throwaway SSH key and a serial console.

## Result

Run 45993449402 (commit f7504b6): 21 of 23 checks passed, and the two that
failed were wrong in the check itself.  The job took 75 minutes, 67 of them
in the build step, most of it building Emacs from master and the Lix of the
flake.  nixos-anywhere copied the 39.5 GiB closure and installed it in 7
minutes, and galago reached `graphical.target` 13.4 seconds after booting
from its disk, with `systemctl is-system-running` reporting `running` and no
failed units.

| Check | Outcome |
| --- | --- |
| Partitions | `disk-disk1-ESP`, `disk-disk1-plainSwap`, and `disk-disk1-root`, in that order |
| ESP | 512M, EFI system partition type, vfat, mounted at `/boot` |
| Swap | 32G, in use, `resume=` on the kernel command line, and named by `/sys/power/resume` |
| Root | rest of the disk, bcachefs with UUID `5b7154e0-3963-19ed-a95b-50c3ae7fa239`, the one in fstab |
| Subvolumes | `@`, `@home`, `@home/yiyu`, `@home/yiyu/.snapshots`, and `@nix` |
| Mounts | `/`, `/nix`, and `/home` from `@`, `@nix`, and `@home`, with `noatime` |
| Home | `/home/yiyu` owned by `yiyu:users`; snapper created snapshot 1 of it |
| Boot | systemd-boot 262.2 installed, with `Linux Boot Manager` in the EFI boot order |
| Compression | FAIL, but the check read `/sys/fs/bcachefs/<uuid>/options/`, which bcachefs 1.39 no longer provides; the superblock printed by `bcachefs format` has `compression: zstd` and `background_compression: zstd`, and the 39.5 GiB closure used 14.3G |

Run 45980593415 (commit c35266e) installed and booted galago the same way,
but passed without running a single check: the check script went to bash on
its stdin, which a command early in the script consumed.  Commit f7504b6
copies the script to the machine first and fails unless it runs to the end.

## Notes

- The mount options, `rw,noatime,verbose`, and the subvolume sources of
  every mount match those of the real galago.  Partition UUIDs differ, since
  disko creates them at random, and fstab names the ESP and swap by
  partition label instead.
- Each boot logs `error reading superblock: Device or resource busy` from
  the bcachefs mount helper a few times, as the parallel mounts of one
  filesystem race; the real galago logs the same.
- Before creating each subvolume, the disko script checks whether it
  already exists by snapshotting it into a directory under `/tmp`, on
  another filesystem, which always fails with `Invalid argument`.  On an
  empty disk this is harmless; on a formatted one, the `test -d` after it is
  what keeps the subvolumes from being created twice.
