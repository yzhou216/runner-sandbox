#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Fixes, in the flake checked out in the given directory, the problems
# that `nix flake check` finds on the base commit, so that a run shows
# whether the check passes once they are gone.  None of these changes
# belongs to the series under test.
#
# Usage: flake-ci/fixups.sh <directory>
set -euo pipefail

cd "$1"
# flake.nix has lines too long for nixfmt; this runs the formatter
# output under test
nix fmt
# iris.nix takes no module arguments, so the module system cannot call it
sed -i 's/^{ }:$/{ ... }:/' machines/darwin/iris.nix
# The OpenWrt package indexes change daily, and so do their hashes
nix flake update openwrt-imagebuilder
git add -A
git commit --quiet -m fixups
git show --stat HEAD | cat
