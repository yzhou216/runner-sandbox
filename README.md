<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# galago's disko layout, installed with nixos-anywhere

Tests whether the disk layout of the `galago` machine in
[yzhou216/flake](https://github.com/yzhou216/flake) can be reproduced on an
empty disk with nixos-anywhere, first as `disko/bcachefs.nix` stands at
commit 7f2b444, then with the root encrypted by bcachefs and unlocked in the
initrd.  The jobs run on ubuntu-latest, under KVM, after `nothing-but-nix`
frees disk space for `/nix`.

`galago-disko/test.sh` runs each phase as a step of its own:

1. It builds galago, its disko script, and the `bcachefs-iso` installer of
   the flake.
2. It boots the installer in QEMU with OVMF and an empty 100G NVMe disk, so
   that the default device of the layout, `/dev/nvme0n1`, applies unchanged.
3. It runs nixos-anywhere with `--store-paths` against the installer over
   SSH, with all of its default phases, up to the reboot, and with
   `--disk-encryption-keys` for a throwaway passphrase.
4. It boots the virtual machine again from the NVMe disk alone, answers each
   passphrase prompt of the initrd on the serial console, and fails unless
   there is exactly one.
5. It runs `galago-disko/check.sh` as root on the installed system.
6. It installs the same system with the stock unlock units of NixOS as a
   second generation, boots it, and counts its prompts.

## Changes for the test

`galago-disko/flake.patch` holds the changes under test as a diff against
7f2b444, which are now the `hotfix/galago-bcachefs-encryption` branch of the
flake apart from an unrelated commit:

- 6de2201 through 09224bd share the nix settings between the machines and
  the installer, and are on the flake's master.
- 611e012 patches the disko module with the commit of
  [nix-community/disko#1316](https://github.com/nix-community/disko/pull/1316),
  fetched by URL, which fixes how disko tells that an encrypted bcachefs
  filesystem on a single device is unlocked; see below.  Up to run
  46081950969, the workflow applied the same fix through a lock override.
- aaa0608 switches galago to `disko/bcachefs-encrypted.nix`, which encrypts
  the bcachefs filesystem, encrypts swap with dm-crypt and a key made anew at
  each boot, turns hibernation off, and keeps only the initrd unlock unit for
  `/`.

The other changes leave the disk layout alone:

- `galago-disko/galago.nix` replaces `linuxPackages_testing` with
  `linuxPackages_latest`, 7.3.6, and drops the xhci patch: at 7f2b444,
  galago does not build, since the patch no longer applies to 7.4-rc4,
  which carries an equivalent fix, and the bcachefs 1.39.7 module fails to
  compile against 7.4.  It also moves the console to the serial port, where
  the test answers the passphrase prompt.
- `galago-disko/installer.nix` disables the COSMIC desktop and Firefox,
  whose dependencies were not yet cached, and adds a throwaway SSH key and a
  serial console.

## Result

Runs 46081950969 (commit a2dc3f3) and 46091061640 (commit 81814c8) passed
all 25 checks, the second with the flake patching disko itself; both built
the same derivations of galago and of its disko script.  galago asked for
the passphrase once, and its unlock unit logged
`Device is encrypted and locked` before unlocking `/`; `/nix` and `/home`
mounted without asking.

| Check | Outcome |
| --- | --- |
| Partitions | `disk-disk1-ESP`, `disk-disk1-plainSwap`, and `disk-disk1-root`, in that order |
| ESP | 512M, EFI system partition type, vfat, mounted at `/boot` |
| Swap | 32G, nothing readable on the partition, in use through `CRYPT-PLAIN` dm-crypt |
| Hibernation | no `resume=`, `/sys/power/resume` at `0:0`, and `CanHibernate` answers `no` |
| Root | rest of the disk, bcachefs with UUID `5b7154e0-3963-19ed-a95b-50c3ae7fa239`, encrypted |
| Compression | `zstd` and background `zstd` |
| Subvolumes | `@`, `@home`, `@home/yiyu`, `@home/yiyu/.snapshots`, and `@nix` |
| Mounts | `/`, `/nix`, and `/home` from `@`, `@nix`, and `@home`, with `noatime` |
| Home | `/home/yiyu` owned by `yiyu:users`; snapper created a snapshot of it |
| Boot | systemd-boot installed; `systemctl is-system-running` reports `running` |

With the stock unlock units, the same system asked for the passphrase twice
on its boot, and the count varies.  NixOS starts one unit for each of `/`,
`/nix`, and `/home`, and `bcachefs unlock` opens the device exclusively, so
the units collide: the one for `/home` got EBUSY in its check and was
skipped, the one for `/nix` asked first and then failed with EBUSY, and the
one for `/` asked again and unlocked.

### Runs

| Run | Commit | Outcome |
| --- | --- | --- |
| 45980593415 | c35266e | Unencrypted layout installed and booted, but no check ran: the script went to bash on its stdin, which a command early in it consumed |
| 45993449402 | f7504b6 | Unencrypted layout: 21 of 23 checks; the compression checks read `/sys/fs/bcachefs/<uuid>`, which bcachefs 1.39 names after the device instead |
| 46068148242 | 22f2456 | Encrypted layout: disko stopped at the second mount, as below |
| 46072716624 | 51a41fb | Installed with the disko fix; the boot got 8 of the 17 characters of the passphrase, since QEMU drops what the guest has not read when the client hangs up |
| 46077764939 | 5d3ece5 | Cancelled: its checks ran `bcachefs unlock -c` on the mounted root, which cannot read its superblock |
| 46081950969 | a2dc3f3 | Passed, as above |
| 46091061640 | 81814c8 | Passed, with the flake fetching the disko fix from the pull request |

## disko's bcachefs test

After a subvolume of an encrypted filesystem is mounted, disko runs
`bcachefs unlock` again for the next one, which fails with EBUSY on the open
device, and then takes `/sys/fs/bcachefs/<uuid>` as proof that the
filesystem is unlocked.  bcachefs names that directory after the UUID only
for a multi-device filesystem, and after the device otherwise, so the
install of an encrypted single-device filesystem with more than one mounted
subvolume fails.  `disko-bcachefs/disko.patch` also accepts a mount of the
filesystem as proof, and adds such a filesystem to the bcachefs example and
test of disko.  `.github/workflows/disko-bcachefs.yml` runs that test on
disko's master with the patch, and without its fix.

| Run | Commit | Outcome |
| --- | --- | --- |
| 46082173056 | c32f0c9 | Without the fix, the test failed at the second mount, as it should; with it, the system installed but did not boot, since its two mounts of the new filesystem were siblings, and one opened the device while an unlock unit held it |
| 46087932691 | fe03391 | With the second mount beneath the first, the system booted twice with the fix, but the test looked up `/srv` as the device `srv]`, from findmnt's `/dev/vde1[/subvolumes/srv]` |
| 46088390189 | 1bfa1f2 | Passed twice with the fix, and failed at the second mount without it |

The change is proposed as
[nix-community/disko#1316](https://github.com/nix-community/disko/pull/1316).

## disko's bcachefs device without a label

The label of a bcachefs device in disko defaults to `""`, but disko passes
`--label=` to `bcachefs format` even then, which bcachefs-tools 1.39.7
rejects with `error creating disk path: Invalid argument (os error 22)`.
`disko-bcachefs-label/disko.patch` passes `--label` only when a label is
set, and drops the label of `vde1` from the bcachefs example of disko.
`.github/workflows/disko-bcachefs-label.yml` runs the bcachefs test of disko
on its master with the patch, and without its fix.

| Run | Commit | Outcome |
| --- | --- | --- |
| 46122091391 | 6c3f195 | Passed with the fix, where sysfs reports the label of `vde1` as `none`; failed without it, when formatting `vde1` |

The change is proposed as
[nix-community/disko#1317](https://github.com/nix-community/disko/pull/1317).

## Notes

- The serial console takes input in chunks of 8 bytes, the trigger level of
  the UART that Linux sets, and QEMU drops the rest of the input once the
  client of its socket hangs up; the test stays connected for five seconds
  after typing.
- bcachefs-tools 1.39 cannot read the superblock of a mounted device, so the
  check takes the encryption of root from the output of the initrd unlock
  unit, which runs before the mount.
- Mount options of bcachefs apply to the whole filesystem: `verbose`, set on
  `@` alone, shows on `/nix` and `/home` too.
- The build step of the galago runs took 23 to 67 minutes, most of it on
  Emacs from master and the Lix of the flake.
