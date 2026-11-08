#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Build the test programs for one processor, each as it is (_ST) and with
# the shim linked in (_SH), under names that fit ISO 9660.
#
#   build.sh <x86_64|i686> <output directory>
set -euo pipefail

arch=$1
out=$(realpath -m "$2")
here=$(dirname "$(realpath "$0")")
target=$arch-pc-windows-gnullvm
cc=$arch-w64-mingw32-clang
var=CARGO_TARGET_$(echo "$target" | tr a-z- A-Z_)
unwind=UNWIND_$arch
export "${var}_LINKER=$cc" "CC_${target//-/_}=$cc"
export "${var}_RUSTFLAGS=-C target-feature=+crt-static -L ${!unwind}"
export CINQ_SHARE_DIR= # No manual pages, which need scdoc

shim=$(mktemp -d)
$cc -O2 -Wall -Wextra -Werror -c "$here/shim/rosshim.c" -o "$shim/rosshim.o"
$arch-w64-mingw32-ar rcs "$shim/librosshim.a" "$shim/rosshim.o"

# Program $2 of the package in directory $1, saved under the name $3.  The
# shim goes in whole and first, so that its definitions win over the import
# libraries that the crates link.
build() {
  local exe=$here/$1/target/$target/release/$2.exe
  (cd "$here/$1" && cargo rustc --release --target "$target" --bin "$2")
  cp "$exe" "$out/${3}_ST.EXE"
  (cd "$here/$1" && cargo rustc --release --target "$target" --bin "$2" -- \
    -L "$shim" -l static:+whole-archive=rosshim -l advapi32 -l shell32 -l ole32)
  cp "$exe" "$out/${3}_SH.EXE"
}

mkdir -p "$out"
build proj hello HELLO
build proj one ONE
build proj threads THRDS
build proj counter COUNT
build cinq cinq CINQ

# Programs in C that run the tests inside ReactOS and check its TLS
$cc -O2 -Wall -Wextra -Werror "$here/vm/runner.c" -o "$out/RUNNER.EXE"
$cc -O2 -Wall -Wextra -Werror "$here/vm/tlsprobe.c" -o "$out/TLSPROBE.EXE"
