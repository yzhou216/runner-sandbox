#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Builds riscv64 derivations longer than a runner's 6-hour job in a codespace
# prepared by setup.sh, from the binary cache that riscv64-export.yml uploaded
# in run EXPORT_RUN.  Then uploads their closure as NAME.tar, a file binary
# cache, to the riscv64-import release for riscv64-qemu.yml to import, and
# stops the codespace.  It first built the stdenv's final gcc
# (`build.sh RUN gcc stdenv.cc.cc`).
# Usage: build.sh EXPORT_RUN NAME ATTR...
set -euo pipefail
umask 022
run=$1 name=$2
shift 2
attrs=("${@/%/^*}")
repo=yzhou216/runner-sandbox
tag=riscv64-import
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
	echo "[res] $(date -u +%T) mem $(free -m | awk '/^Mem/ { print $7 "M available" }'), swap $(free -m | awk '/^Swap/ { print $3 "M used" }'), disk $(df -h /tmp | awk 'NR == 2 { print $4 " free" }'), load $(cut -d ' ' -f 1-3 /proc/loadavg)"
done &
reslog=$!
trap 'kill "$reslog"' EXIT
date -u
nix build -L --no-link --argstr system riscv64-linux -f . "${attrs[@]}"
date -u
nix path-info --argstr system riscv64-linux -f . "${attrs[@]}"

rm -rf /tmp/export
nix copy --to 'file:///tmp/export?compression=zstd' --argstr system riscv64-linux -f . "${attrs[@]}"
echo "$(find /tmp/export -name '*.narinfo' | wc -l) paths exported, $(du -sh /tmp/export | cut -f1)"
tar -C /tmp -cf "/tmp/$name.tar" export
# A new release pushes a tag, which runs the push workflows of the commit it
# points at; the default branch has none
gh release view "$tag" -R "$repo" > /dev/null 2>&1 ||
	gh release create "$tag" -R "$repo" --target master --prerelease \
		--title "riscv64 store paths built in a codespace" \
		--notes "File binary caches of riscv64 store paths that took longer than a runner's job, for riscv64-qemu.yml to import.  To be deleted at the end of the session."
gh release upload "$tag" "/tmp/$name.tar" --clobber -R "$repo"
echo "uploaded $name.tar"

# Stop the billing as soon as the upload is done
gh codespace stop -c "$CODESPACE_NAME" ||
	echo "could not stop $CODESPACE_NAME, so stop it from outside"
