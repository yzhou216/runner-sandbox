# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

{ inputs, ... }:
{
  perSystem =
    {
      config,
      lib,
      pkgs,
      crate,
      sourceDateEpoch,
      installBundle,
      ...
    }:
    let
      # Rust's own toolchain of the release that Nixpkgs builds rustc
      # from, with the standard libraries for Apple silicon and Windows,
      # which Nixpkgs cannot build on GNU/Linux
      rust = (inputs.rust-overlay.lib.mkRustBin { } pkgs).stable.${pkgs.rustc.version}.minimal.override {
        targets = [
          "aarch64-apple-darwin"
          "aarch64-pc-windows-gnullvm"
          "x86_64-pc-windows-gnullvm"
        ];
      };
      # cinq that cargo cross-builds for `platform`, a platform of Nixpkgs,
      # with the variables of `env`, and that `install` lays out from the
      # directory that the build leaves the program in, named after the
      # system that it runs on
      crossBuilt =
        {
          platform,
          env,
          install,
        }:
        let
          target = platform.rust.rustcTarget;
        in
        pkgs.stdenv.mkDerivation {
          pname = "${crate.name}-${platform.system}";
          inherit (crate) version;
          inherit (config.packages.cinq) src cargoDeps;

          nativeBuildInputs = [
            rust
          ]
          ++ (with pkgs; [
            rustPlatform.cargoSetupHook
            scdoc
          ]);

          env = env // {
            SOURCE_DATE_EPOCH = sourceDateEpoch;
          };

          buildPhase = ''
            runHook preBuild
            cargo build --frozen --release --target ${target}
            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            ${install "target/${target}/release"}
            runHook postInstall
          '';
        };
      # cinq for Windows on the processor of `cross`, the Nixpkgs of a
      # MinGW system, whose LLVM links the program.  The C runtime and the
      # unwinder go into the program, so that it needs no DLL that
      # Windows lacks.
      windows =
        cross:
        let
          inherit (cross.stdenv) cc hostPlatform;
          underscored = builtins.replaceStrings [ "-" ] [ "_" ] hostPlatform.rust.rustcTarget;
          variable = "CARGO_TARGET_${lib.toUpper underscored}";
        in
        crossBuilt {
          platform = hostPlatform;
          env = {
            "${variable}_LINKER" = "${cc}/bin/${cc.targetPrefix}cc";
            "${variable}_RUSTFLAGS" = "-C target-feature=+crt-static";
          };
          install = release: ''install -D ${release}/cinq.exe "$out/bin/cinq.exe"'';
        };
    in
    {
      # cinq for other systems, each named after the system that it runs on,
      # while the system of the flake is the one that builds it
      packages = lib.optionalAttrs pkgs.stdenvNoCC.hostPlatform.isLinux {
        # cinq for Apple silicon, laid out as the package for Darwin is,
        # which cargo cross-builds against the SDK that Nixpkgs fetches
        # from Apple, linked by lld from the toolchain
        aarch64-darwin = crossBuilt {
          platform = lib.systems.elaborate "aarch64-darwin";
          env = {
            SDKROOT = "${pkgs.apple-sdk.src}";
            CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER = "rust-lld";
          };
          install = release: ''
            install -D ${release}/cinq "$out/bin/cinq"
            cp -r target/share "$out"
            ${installBundle}
          '';
        };
        x86_64-windows = windows pkgs.pkgsCross.mingw-ucrt-x86_64-llvm;
        aarch64-windows = windows pkgs.pkgsCross.mingw-ucrt-aarch64;
      };
    };
}
