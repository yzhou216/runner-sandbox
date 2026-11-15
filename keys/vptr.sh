#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Builds vptr from vptr.c into DIRECTORY, with wayland-scanner making its
# header and glue from the XML of wlr-protocols, and gcc from Nix
#
# Usage: vptr.sh DIRECTORY, with gcc and wayland-scanner on PATH
set -euo pipefail

here=$(dirname "$(readlink -f "$0")")
dir=$(mkdir -p "$1" && readlink -f "$1")
nix_path() { nix build --inputs-from "$here/../cinq" --no-link --print-out-paths "nixpkgs#$1" | head -n 1; }
xml=$(find "$(nix_path wlr-protocols)" -name 'wlr-virtual-pointer-unstable-v1.xml' | head -n 1)
include=$(nix_path wayland.dev)/include
library=$(nix_path wayland.out)/lib
wayland-scanner client-header "$xml" "$dir/wlr-virtual-pointer-unstable-v1-client-protocol.h"
wayland-scanner private-code "$xml" "$dir/wlr-virtual-pointer-unstable-v1-protocol.c"
gcc -O1 -o "$dir/vptr" -I"$dir" -I"$include" "$here/vptr.c" "$dir/wlr-virtual-pointer-unstable-v1-protocol.c" -L"$library" -Wl,-rpath,"$library" -lwayland-client
