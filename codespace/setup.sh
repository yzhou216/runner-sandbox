#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Prepares a codespace to carry on with the riscv64 build of riscv64-qemu.yml
# the way its runners do: single-user Nix with its store on /tmp (the large
# disk), nixpkgs at REV, and Nixpkgs' static QEMU behind qemu/binfmt-stack.c,
# registered with the F flag and reachable from Nix's sandbox.  Run it from
# the codespace's checkout of this branch.
# Usage: setup.sh REV
set -euo pipefail
rev=$1
src=$(dirname "$(readlink -f "$0")")/../qemu/binfmt-stack.c
# /tmp carries ACLs that make new files group- and world-writable, which Nix
# rejects in build outputs
umask 022

# gcc's parallel links under QEMU may outgrow the 16 GiB of memory
if ! swapon --show | grep -q /tmp/swapfile; then
	sudo fallocate -l 16G /tmp/swapfile
	sudo chmod 600 /tmp/swapfile
	sudo mkswap -q /tmp/swapfile
	sudo swapon /tmp/swapfile
fi

command -v setfacl > /dev/null || { sudo apt-get update -q && sudo apt-get install -y -q acl; }
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
[ -e ~/.nix-profile/bin/nix ] || sh <(curl -fsSL https://nixos.org/nix/install) --no-daemon --yes
# shellcheck source=/dev/null
. ~/.nix-profile/etc/profile.d/nix.sh
nix --version

if [ ! -d /tmp/nixpkgs/.git ]; then
	git init -q /tmp/nixpkgs
	git -C /tmp/nixpkgs fetch -q --depth 1 https://github.com/yzhou216/nixpkgs "$rev"
	git -C /tmp/nixpkgs checkout -q FETCH_HEAD
fi
git -C /tmp/nixpkgs log --oneline -1

# As in riscv64-qemu.yml: Ubuntu's QEMU 8.2 has QEMU issue 2448, and QEMU
# takes the chain's `ulimit -s unlimited` to mean an 8 MiB stack
out=$(nix build --no-link --print-out-paths -f /tmp/nixpkgs pkgsStatic.qemu-user)
"$out/bin/qemu-riscv64" --version
wrapper=/usr/local/libexec/qemu-riscv64-stack
sudo mkdir -p "${wrapper%/*}"
sudo gcc -static -O2 -Wall -Werror -DQEMU="\"$out/bin/qemu-riscv64\"" -DSTACK='"4000M"' \
	-o "$wrapper" "$src"
mountpoint -q /proc/sys/fs/binfmt_misc ||
	sudo mount -t binfmt_misc binfmt_misc /proc/sys/fs/binfmt_misc
[ ! -e /proc/sys/fs/binfmt_misc/qemu-riscv64 ] ||
	echo -1 | sudo tee /proc/sys/fs/binfmt_misc/qemu-riscv64 > /dev/null
magic='\x7fELF\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02\x00\xf3\x00'
mask='\xff\xff\xff\xff\xff\xff\xff\x00\xff\xff\xff\xff\xff\xff\xff\xff\xfe\xff\xff\xff'
echo ":qemu-riscv64:M::$magic:$mask:$wrapper:PF" | sudo tee /proc/sys/fs/binfmt_misc/register > /dev/null
cat /proc/sys/fs/binfmt_misc/qemu-riscv64
echo "extra-sandbox-paths = $out" >> ~/.config/nix/nix.conf
df -h / /tmp
