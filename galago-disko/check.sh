#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Runs as root on the installed galago and checks the layout that
# disko/bcachefs-encrypted.nix describes.  The only argument is the
# store path of the system that nixos-anywhere installed.

set -uo pipefail
exec </dev/null

toplevel=$1
uuid=5b7154e0-3963-19ed-a95b-50c3ae7fa239
esp=/dev/disk/by-partlabel/disk-disk1-ESP
swap=/dev/disk/by-partlabel/disk-disk1-plainSwap
root=/dev/disk/by-partlabel/disk-disk1-root
# Named after the device in bcachefs 1.39, not after the UUID
options=/sys/fs/bcachefs/$(basename "$(readlink -f $root)")/options
failed=0

check() {
  local what=$1
  shift
  if "$@"; then
    echo "PASS: $what"
  else
    echo "FAIL: $what"
    failed=1
  fi
}

report() {
  echo "\$ $*"
  "$@" 2>&1
  echo
}

# Device-mapper device that swap is encrypted through
crypt=$(lsblk -nro NAME,TYPE $swap | awk '$2 == "crypt" { print $1 }')
dm=$(basename "$(readlink -f "/dev/mapper/$crypt")")

# Mount the root of the filesystem to see every subvolume
mkdir -p /run/fsroot
mount -t bcachefs "/dev/disk/by-uuid/$uuid" /run/fsroot

echo "== Report"
report uname -r
report cat /proc/cmdline
report lsblk -o NAME,SIZE,TYPE,PARTLABEL,PARTTYPENAME,FSTYPE,UUID,MOUNTPOINTS /dev/nvme0n1
report blkid
report findmnt -o TARGET,SOURCE,FSTYPE,OPTIONS -t bcachefs,vfat
report swapon --show
report cat "/sys/block/$dm/dm/uuid" /sys/power/resume
# bcachefs-tools cannot read the superblock of a mounted device, so take
# the state of the filesystem before it was mounted from the journal
report journalctl -b -o cat --no-pager -u unlock-bcachefs--.service
report grep -r . "$options/compression" "$options/background_compression"
report bcachefs subvolume list -R /run/fsroot
report bcachefs fs usage -h /
report stat -c '%A %U:%G %n' /home /home/yiyu /home/yiyu/.snapshots
report cat /etc/systemd/sleep.conf
report busctl call org.freedesktop.login1 /org/freedesktop/login1 \
  org.freedesktop.login1.Manager CanHibernate
report journalctl -b -o short-monotonic --no-pager --grep 'nlock bcachefs'
report bootctl status --no-pager
report systemctl is-system-running --wait
report systemctl --failed --no-pager
report snapper -c home create --description vm-check
report snapper -c home list

echo "== Checks"
size() { lsblk -bdno SIZE "$1"; }
fstype() { lsblk -dno FSTYPE "$1"; }
subvolumes=$(bcachefs subvolume list -R /run/fsroot |
  awk '$1 != "Path" { sub("^/run/fsroot/", "", $1); print $1 }')

# shellcheck disable=SC2329 # Called through check
swap_through_plain_crypt() {
  [ -n "$crypt" ] &&
    [[ $(cat "/sys/block/$dm/dm/uuid") == CRYPT-PLAIN-* ]] &&
    swapon --show=NAME --noheadings | grep -qx "/dev/$dm"
}

# shellcheck disable=SC2329 # Called through check
root_was_locked() {
  journalctl -b -o cat --no-pager -u unlock-bcachefs--.service |
    grep -qx 'Device is encrypted and locked'
}

check "installed system is the one built" \
  test "$(readlink -f /run/current-system)" = "$toplevel"
check "partitions in order ESP, swap, root" \
  test "$(lsblk -no PARTLABEL /dev/nvme0n1 | sed '/^$/d' | paste -sd ' ')" \
  = "disk-disk1-ESP disk-disk1-plainSwap disk-disk1-root"
check "ESP is a 512M EFI system partition" \
  test "$(size $esp)" = $((512 << 20)) -a "$(lsblk -dno PARTTYPE $esp)" = c12a7328-f81f-11d2-ba4b-00a0c93ec93b
check "ESP is vfat at /boot" \
  test "$(fstype $esp)" = vfat -a "$(findmnt -no SOURCE /boot)" = "$(readlink -f $esp)"
check "swap partition is 32G, with nothing readable on it" \
  test "$(size $swap)" = $((32 << 30)) -a -z "$(blkid -p -o value -s TYPE $swap)"
check "swap in use through dm-crypt with a random key" swap_through_plain_crypt
check "kernel has no resume device" \
  test -z "$(grep -o 'resume=' /proc/cmdline)" -a "$(cat /sys/power/resume)" = 0:0
check "logind refuses to hibernate" \
  test "$(busctl call org.freedesktop.login1 /org/freedesktop/login1 \
    org.freedesktop.login1.Manager CanHibernate)" = 's "no"'
check "root partition fills the rest of the disk" \
  test $(($(size /dev/nvme0n1) - $(size $esp) - $(size $swap) - $(size $root))) -lt $((4 << 20))
check "root is bcachefs with the UUID in fstab" \
  test "$(fstype $root)" = bcachefs -a "$(lsblk -dno UUID $root)" = "$uuid"
check "initrd found root encrypted by bcachefs and locked" root_was_locked
check "compression is zstd" test "$(cat "$options/compression")" = zstd
check "background compression is zstd" \
  test "$(cat "$options/background_compression")" = zstd
check "initrd unlocked bcachefs for / alone" \
  test "$(journalctl -b -o cat --no-pager --grep '^Finished .*Unlock bcachefs for ' |
    sed 's/.*Unlock bcachefs for //; s/\.$//' | paste -sd ' ')" = /
for subvolume in @ @nix @home @home/yiyu @home/yiyu/.snapshots; do
  check "subvolume $subvolume exists" grep -qx -- "$subvolume" <<<"$subvolumes"
done
for mount in /:@ /nix:@nix /home:@home; do
  check "${mount%%:*} is subvolume ${mount#*:} with noatime" \
    test "$(findmnt -no SOURCE "${mount%%:*}")" = "$(readlink -f $root)[/${mount#*:}]" \
    -a -n "$(findmnt -no OPTIONS "${mount%%:*}" | tr , '\n' | grep -x noatime)"
done
check "yiyu owns /home/yiyu" test "$(stat -c %U /home/yiyu)" = yiyu
check "snapper takes snapshots of /home/yiyu" \
  test -n "$(snapper -c home list | grep -w vm-check)"
check "systemd-boot is installed" bootctl is-installed --quiet

umount /run/fsroot
echo "== Done"
exit $failed
