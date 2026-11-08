#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# List the imports of programs that a ReactOS boot CD cannot resolve, which
# its loader refuses to start.  API sets resolve through the table that
# ReactOS's ntdll holds, taken from the source of the same revision.
#
#   check-imports.sh <boot CD> <i386|amd64> <git revision> <program...>
set -euo pipefail

iso=$(realpath "$1")
arch=$2
revision=$3
shift 3
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

7zz x -y -bso0 -bsp0 -o"$work/iso" "$iso" "$arch/system32" "$arch/reactos.cab"
cabextract -q -d "$work/dlls" "$work/iso/$arch/reactos.cab"
cp "$work/iso/$arch/system32/"*.dll "$work/dlls/"
mkdir "$work/exports"
for dll in "$work/dlls/"*.dll; do
  name=$(basename "${dll,,}")
  llvm-readobj --coff-exports "$dll" | awk '/^  Name: / {print $2}' \
    > "$work/exports/$name"
done
curl -fsSL "https://raw.githubusercontent.com/reactos/reactos/$revision/sdk/lib/apisets/apisets.table.c" \
  | sed -nE 's/.*L"(api-ms-[^"]+)"\), RTL_CONSTANT_STRING\(L"([^"]+)"\).*/\1\t\2/p' \
  | tr A-Z a-z > "$work/apisets"

for program in "$@"; do
  llvm-readobj --coff-imports "$program" \
    | awk '/^  Name: / {dll = tolower($2)} /^  Symbol: / {print dll "\t" $2}' \
    | sort -u \
    | while IFS=$'\t' read -r dll symbol; do
        host=$dll
        case $dll in
          api-ms-*) host=$(awk -F'\t' -v set="${dll%.dll}" '$1 == set {print $2}' "$work/apisets") ;;
        esac
        if [ ! -f "$work/exports/$host" ]; then
          echo "$dll!$symbol (no $host)"
        elif ! grep -qxF "$symbol" "$work/exports/$host"; then
          echo "$dll!$symbol"
        fi
      done > "$work/missing"
  echo "$(basename "$program"): $(wc -l < "$work/missing") missing"
  sed 's/^/  /' "$work/missing"
done
