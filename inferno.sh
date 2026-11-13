#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Builds Inferno, an unofficial implementation of Dante that is not endorsed
# by Audinate, at a pinned commit: its ALSA plugin, which makes a program
# that plays on it a Dante device, and the fake clock server of its own
# tests, which stands in for PTP.  The plugin is built with the toolchain
# and the ALSA library of cinq's development shell, so that cinq can load
# it.
#
# Usage: inferno.sh CINQ OUT, with CINQ the tree of cinq and OUT the
# directory for libasound_module_pcm_inferno.so and fake_usrvclock_server
set -euo pipefail
cinq=$(readlink -f "$1")
out=$(mkdir -p "$2" && readlink -f "$2")
commit=9767558e0905c9e9c960f5a60e050fbdc38300b7
src=$(mktemp -d)/inferno
git clone --quiet https://github.com/teodly/inferno.git "$src"
git -C "$src" checkout --quiet "$commit"
git -C "$src" submodule --quiet update --init --recursive
# Tells apps that poll before they write, as cpal does, that a playback
# stream just prepared can take a buffer
git -C "$src" apply "$(dirname "$(readlink -f "$0")")/inferno-prepare-pollout.patch"
# Profile roots the shell, which a collection of garbage on a full disk
# would otherwise take away midway
(cd "$src" && nix develop "$cinq" --profile "$src/shell" -c cargo build --release --locked -p alsa_pcm_inferno)
cp "$src/target/release/libasound_module_pcm_inferno.so" "$out/"
gcc -O2 -o "$out/fake_usrvclock_server" \
  "$src/test/dockerized_trx/fake_usrvclock_server/fake_usrvclock_server.c"
