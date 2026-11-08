# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

{
  description = "Rust, iced, and cinq on ReactOS";

  inputs = {
    # cinq's own pins at f48fe08
    nixpkgs.url = "github:NixOS/nixpkgs/d357a2b54f0b6dc4aeb4e993190a52cf151b6f24";
    rust-overlay = {
      url = "github:oxalica/rust-overlay/6b29b8e8d3cc8ebda9342fbbcc7ac06eff58e2fb";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      # MinGW systems with LLVM and the UCRT, as cinq's Windows build uses;
      # Nixpkgs has none for i686, so that one builds from source
      cross = {
        x86_64 = pkgs.pkgsCross.mingw-ucrt-x86_64-llvm;
        i686 = import nixpkgs {
          localSystem = "x86_64-linux";
          crossSystem = {
            config = "i686-w64-mingw32";
            libc = "ucrt";
            useLLVM = true;
          };
        };
      };
      rust = (rust-overlay.lib.mkRustBin { } pkgs).stable.${pkgs.rustc.version}.minimal.override {
        targets = map (arch: "${arch}-pc-windows-gnullvm") (builtins.attrNames cross);
      };
      # libunwind alone in a directory, since the cc wrapper's -lunwind
      # otherwise takes its DLL
      unwind =
        arch:
        pkgs.runCommand "libunwind-static-${arch}" { } ''
          mkdir -p $out
          ln -s ${cross.${arch}.llvmPackages.libunwind}/lib/libunwind.a $out/
        '';
      build =
        arch:
        pkgs.mkShell {
          nativeBuildInputs = [
            rust
            cross.${arch}.stdenv.cc
          ];
          "UNWIND_${arch}" = unwind arch;
          ROSTEST_FONT = "${pkgs.dejavu_fonts}/share/fonts/truetype/DejaVuSans.ttf";
        };
    in
    {
      devShells.x86_64-linux = {
        x86_64 = build "x86_64";
        i686 = build "i686";
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
