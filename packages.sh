#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Builds deb, rpm, and archlinux from cinq-before and cinq-after, unpacks
# each, and fails when the files inside differ.
set -uo pipefail
status=0
for variant in before after; do
  for output in deb rpm archlinux; do
    nix build "./cinq-$variant#$output" -o "result-$variant-$output" || exit 1
    dir=unpacked-$variant-$output
    mkdir -p "$dir"
    file=$(echo "result-$variant-$output"/*)
    case $output in
      deb) nix shell --inputs-from ./cinq-after nixpkgs#dpkg -c dpkg-deb -x "$file" "$dir" ;;
      *) nix shell --inputs-from ./cinq-after nixpkgs#libarchive -c bsdtar -xf "$file" -C "$dir" ;;
    esac
  done
done
for output in deb rpm archlinux; do
  echo "== $output"
  sha256sum result-before-$output/* result-after-$output/*
  cmp result-before-$output/* result-after-$output/* && echo "package files identical"
  nix shell --inputs-from ./cinq-after nixpkgs#patchelf -c bash -c \
    "patchelf --print-interpreter unpacked-before-$output/usr/bin/cinq unpacked-after-$output/usr/bin/cinq"
  diff -r --no-dereference "unpacked-before-$output" "unpacked-after-$output" && echo "files inside identical" || status=1
done
exit $status
