#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Prints the loader that the packages of system $1 set as the interpreter of
# cinq, as flake/distro.nix chose it before and as it chooses it now.  The
# new name is the one of the linker's own file, which is what `patchelf
# --print-interpreter` shows for a program that linker wrote.  Exits nonzero
# when the two differ and $2 is "same".
set -uo pipefail
sys=$1 expect=${2:-}
nixpkgs=nixpkgs#legacyPackages.$sys
eval_() { nix eval --inputs-from ./cinq-before "$@"; }
glob=$(eval_ --raw "$nixpkgs.stdenv.cc.bintools.dynamicLinker")
x86=$(eval_ "$nixpkgs.stdenv.hostPlatform.isx86_64")
power=$(eval_ "$nixpkgs.stdenv.hostPlatform.isPower64")
old=/$([ "$x86" = true ] && echo lib64 || echo lib)/$(basename "$glob")
new=unavailable
if paths=$(nix build --inputs-from ./cinq-before --max-jobs 0 --no-link --print-out-paths "$nixpkgs.stdenv.cc.bintools"); then
  for p in $paths; do
    [ -f "$p/nix-support/dynamic-linker" ] && new=/$([ "$x86" = true ] || [ "$power" = true ] && echo lib64 || echo lib)/$(basename "$(cat "$p/nix-support/dynamic-linker")")
  done
fi
echo "$sys: glob $glob"
echo "$sys: before $old"
echo "$sys: after  $new"
[ "$expect" != same ] || [ "$old" = "$new" ]
