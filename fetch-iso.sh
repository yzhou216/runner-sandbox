#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Download a ReactOS boot CD and print its path.
#
#   fetch-iso.sh <0.4.16-x86|nightly-x86|nightly-x64> <directory>
set -euo pipefail

nightly=https://iso.reactos.org/bootcd/reactos-bootcd-0.4.17-dev-1340-gfaa2d03
case $1 in
  0.4.16-x86) url=https://downloads.sourceforge.net/project/reactos/ReactOS/0.4.16/ReactOS-0.4.16-i386.zip ;;
  nightly-x86) url=$nightly-x86-gcc-lin-dbg.7z ;;
  nightly-x64) url=$nightly-x64-msvc-win-dbg.7z ;;
  *) echo "unknown image $1" >&2; exit 2 ;;
esac
mkdir -p "$2"
cd "$2"
curl -fsSL --retry 5 -o archive "$url"
7zz x -y -bso0 -bsp0 archive '*.iso'
rm archive
realpath ./*.iso
