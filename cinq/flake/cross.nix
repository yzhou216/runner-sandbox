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
      system,
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
      # cinq that cargo cross-builds for `platform`, which `install` lays
      # out from the directory that the build leaves the program in
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
          variable = "CARGO_TARGET_${hostPlatform.rust.cargoEnvVarTarget}";
        in
        crossBuilt {
          platform = hostPlatform;
          env = {
            "${variable}_LINKER" = "${cc}/bin/${cc.targetPrefix}cc";
            "${variable}_RUSTFLAGS" = "-C target-feature=+crt-static";
          };
          install = release: ''install -D ${release}/cinq.exe "$out/bin/cinq.exe"'';
        };
      # cinq that `cross`, the Nixpkgs of a system that Nixpkgs builds
      # Rust's standard library for, cross-builds from the flake's package
      nixpkgsCross = cross: cross.callPackage ./cinq.nix { inherit crate sourceDateEpoch installBundle; };
      # Nixpkgs of FreeBSD on the processor `cpu`.  FreeBSD's libraries
      # write their symbol maps with clang's cpp, which Nixpkgs runs for
      # the system that builds them rather than for FreeBSD, and which on
      # another processor rejects FreeBSD's flags and writes nothing, so
      # that libthr lost pthread_create.  There the cpp takes FreeBSD's
      # target, until NixOS/nixpkgs#584780 is fixed.
      freebsd =
        cpu:
        let
          config = "${cpu}-unknown-freebsd";
        in
        if pkgs.stdenv.hostPlatform.parsed.cpu.name == cpu then
          pkgs.pkgsCross."${cpu}-freebsd"
        else
          import pkgs.path {
            localSystem = pkgs.stdenv.hostPlatform;
            crossSystem = { inherit config; };
            crossOverlays = [
              (final: prev: {
                freebsd = prev.freebsd.overrideScope (
                  _: freebsd: {
                    mkDerivation = lib.makeOverridable (
                      attrs:
                      freebsd.mkDerivation (
                        attrs
                        // {
                          # Script rather than flags, since bmake splits
                          # them off into flags of its own
                          env = {
                            CPP = final.buildPackages.writeShellScript "${config}-cpp" ''exec ${config}-cpp --target=${config} "$@"'';
                          }
                          // attrs.env or { };
                        }
                      )
                    );
                  }
                );
              })
            ];
          };
      builds = {
        aarch64-darwin =
          (crossBuilt {
            platform = lib.systems.elaborate "aarch64-darwin";
            env = {
              SDKROOT = "${pkgs.apple-sdk.src}"; # Its package evaluates on Darwin alone
              CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER = "rust-lld";
            };
            install = release: ''
              install -D ${release}/cinq "$out/bin/cinq"
              cp -r target/share "$out"
              ${installBundle}
            '';
          }).overrideAttrs
            # Apple's SDK again once the setup hooks have run on Darwin,
            # where the hook of Nixpkgs' SDK sets an SDK whose libiconv is
            # Nixpkgs', which a Mac without Nix lacks
            (
              lib.optionalAttrs pkgs.stdenv.buildPlatform.isDarwin {
                preBuild = "export SDKROOT=${pkgs.apple-sdk.src}";
              }
            );
        x86_64-windows = windows pkgs.pkgsCross.mingw-ucrt-x86_64-llvm;
        aarch64-windows = windows pkgs.pkgsCross.mingw-ucrt-aarch64;
        x86_64-linux = nixpkgsCross pkgs.pkgsCross.gnu64;
        aarch64-linux = nixpkgsCross pkgs.pkgsCross.aarch64-multiplatform;
        riscv64-linux = nixpkgsCross pkgs.pkgsCross.riscv64;
        x86_64-freebsd = nixpkgsCross (freebsd "x86_64");
        # Rust's tier 3, which Nixpkgs leaves out of the platforms of what
        # Rust builds, though it builds Rust's compiler for it
        aarch64-freebsd = (nixpkgsCross (freebsd "aarch64")).overrideAttrs (old: {
          meta = old.meta // {
            platforms = [ "aarch64-freebsd" ];
          };
        });
      };
    in
    {
      # All but the system's own, whose name stays with its native build
      packages = lib.optionalAttrs pkgs.stdenvNoCC.hostPlatform.isLinux (removeAttrs builds [ system ]);
      # Program of the Darwin package, which links only what every Mac has
      _module.args.darwinBuild = builds.aarch64-darwin;
    };
}
