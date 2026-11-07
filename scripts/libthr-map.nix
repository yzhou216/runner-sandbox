# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# FreeBSD's libthr for x86_64-freebsd, as the Nixpkgs of the flake
# `flake` cross-builds it on `system`, that prints the version map that it
# links with and the tools that write the map
{ flake, system }:
let
  thr = (builtins.getFlake flake).inputs.nixpkgs.legacyPackages.${system}.pkgsCross.x86_64-freebsd.freebsd.libthr;
in
thr.overrideAttrs (old: {
  postBuild = (old.postBuild or "") + ''
    type x86_64-unknown-freebsd-cpp awk
    x86_64-unknown-freebsd-cpp --version | head -1
    awk --version | head -1
    wc -l Version.map
    head -c 4000 Version.map
    echo '#define X 1' | x86_64-unknown-freebsd-cpp -P - | od -c | head -5
  '';
})
