#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Builds the riscv64 stdenv's final gcc, longer than a runner's 6-hour job
# under QEMU, in a codespace prepared by setup.sh, from the binary cache
# that riscv64-export.yml uploaded.  Then uploads gcc's closure as a file
# binary cache in a prerelease asset, for riscv64-qemu.yml to import.
# Usage: build.sh EXPORT_RUN
set -euo pipefail
umask 022
run=$1
repo=yzhou216/runner-sandbox
tag=riscv64-import
gcc='stdenv.cc.cc^*'
# shellcheck source=/dev/null
. ~/.nix-profile/etc/profile.d/nix.sh
GITHUB_TOKEN=$(sed -n 's/^GITHUB_TOKEN=//p' /workspaces/.codespaces/shared/.env)
export GITHUB_TOKEN

# Nix creates an empty cache layout there when it first reads the setting
if [ -z "$(find /tmp/nix-cache -maxdepth 1 -name '*.narinfo' -print -quit 2> /dev/null)" ]; then
	gh run download "$run" -R "$repo" -n nix-cache -D /tmp/dl
	tar -C /tmp -xf /tmp/dl/nix-cache.tar
	rm -rf /tmp/dl
fi
echo "$(find /tmp/nix-cache -name '*.narinfo' | wc -l) paths in the binary cache"

cd /tmp/nixpkgs
# Output every ten minutes, so that the streamed log counts as activity
while sleep 600; do
	echo "[res] $(date -u +%T) mem $(free -m | awk '/^Mem/ { print $7 "M available" }'), disk $(df -h /tmp | awk 'NR == 2 { print $4 " free" }'), load $(cut -d ' ' -f 1-3 /proc/loadavg)"
done &
reslog=$!
trap 'kill "$reslog"' EXIT
date -u
nix build -L --no-link --argstr system riscv64-linux -f . "$gcc"
date -u
nix path-info --argstr system riscv64-linux -f . "$gcc"

nix copy --to 'file:///tmp/export?compression=zstd' --argstr system riscv64-linux -f . "$gcc"
echo "$(find /tmp/export -name '*.narinfo' | wc -l) paths exported, $(du -sh /tmp/export | cut -f1)"
tar -C /tmp -cf /tmp/gcc.tar export
gh release view "$tag" -R "$repo" > /dev/null 2>&1 ||
	gh release create "$tag" -R "$repo" --target ghc-bootstrap/2026-11-23-riscv64-nix --prerelease \
		--title "riscv64 store paths built in a codespace" \
		--notes "File binary cache of riscv64 store paths that took longer than a runner's job, for riscv64-qemu.yml to import.  To be deleted at the end of the session."
gh release upload "$tag" /tmp/gcc.tar --clobber -R "$repo"
echo "uploaded gcc.tar"
