# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

{
  description = "cinq on ReactOS";

  # cinq's own pin, so that the MinGW below is the one that builds cinq
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/d357a2b54f0b6dc4aeb4e993190a52cf151b6f24";

  outputs =
    { nixpkgs, ... }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      # The cross system of cinq's .#i686-windows
      i686 = import nixpkgs {
        localSystem = pkgs.stdenv.hostPlatform;
        crossSystem = {
          config = "i686-w64-mingw32";
          libc = "ucrt";
          useLLVM = true;
          rust.rustcTarget = "i686-pc-windows-gnullvm";
        };
      };
    in
    {
      devShells.x86_64-linux = {
        i686 = pkgs.mkShell { nativeBuildInputs = [ i686.stdenv.cc ]; };
        vm = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            _7zz
            cabextract
            ffmpeg-headless
            llvm
            mtools
            python3
            qemu_kvm
            socat
            util-linux
            xorriso
          ];
        };
      };
    };
}
