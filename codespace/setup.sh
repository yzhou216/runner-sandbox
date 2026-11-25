#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Prepares a codespace to carry on with the riscv64 build of riscv64-qemu.yml:
# Ubuntu's QEMU registered with the F flag, as on the runners, single-user Nix
# with its store on /tmp (the large disk), and nixpkgs at REV.
# Usage: setup.sh REV
set -euo pipefail
rev=$1
# /tmp carries ACLs that make new files group- and world-writable, which Nix
# rejects in build outputs
umask 022

sudo apt-get update -q
sudo apt-get install -y -q qemu-user-static acl
qemu-riscv64-static --version
mountpoint -q /proc/sys/fs/binfmt_misc ||
	sudo mount -t binfmt_misc binfmt_misc /proc/sys/fs/binfmt_misc
[ -e /proc/sys/fs/binfmt_misc/qemu-riscv64 ] ||
	sudo sh -c 'cat /usr/lib/binfmt.d/qemu-riscv64.conf > /proc/sys/fs/binfmt_misc/register'
cat /proc/sys/fs/binfmt_misc/qemu-riscv64
grep -q '^flags: .*F' /proc/sys/fs/binfmt_misc/qemu-riscv64

# gcc's parallel links under QEMU may outgrow the 16 GiB of memory
if ! swapon --show | grep -q /tmp/swapfile; then
	sudo fallocate -l 16G /tmp/swapfile
	sudo chmod 600 /tmp/swapfile
	sudo mkswap -q /tmp/swapfile
	sudo swapon /tmp/swapfile
fi

sudo mkdir -p /tmp/nix /nix
sudo chown "$(id -u):$(id -g)" /tmp/nix
sudo setfacl -b -k /tmp/nix
sudo chmod 755 /tmp/nix
mountpoint -q /nix || sudo mount --bind /tmp/nix /nix
mkdir -p ~/.config/nix
cat > ~/.config/nix/nix.conf << 'EOF'
experimental-features = nix-command
extra-platforms = riscv64-linux
sandbox = true
max-jobs = 2
cores = 4
extra-substituters = file:///tmp/nix-cache?trusted=true
log-lines = 100
EOF
[ -e ~/.nix-profile ] || sh <(curl -fsSL https://nixos.org/nix/install) --no-daemon --yes
# shellcheck source=/dev/null
. ~/.nix-profile/etc/profile.d/nix.sh
nix --version

if [ ! -d /tmp/nixpkgs/.git ]; then
	git init -q /tmp/nixpkgs
	git -C /tmp/nixpkgs fetch -q --depth 1 https://github.com/yzhou216/nixpkgs "$rev"
	git -C /tmp/nixpkgs checkout -q FETCH_HEAD
fi
git -C /tmp/nixpkgs log --oneline -1
df -h / /tmp
