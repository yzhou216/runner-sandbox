# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

{
  description = "Darwin daemon reversing mouse or trackpad scrolling";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.treefmt-nix.flakeModule ];
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      perSystem =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        let
          inherit (pkgs.stdenv.hostPlatform) isDarwin isLinux;
          crate = (lib.importTOML ./Cargo.toml).package;
          sourceDateEpoch = toString inputs.self.lastModified;
          rustfmt = pkgs.rustfmt.override { asNightly = true; };

          # Nixpkgs' Rust release from upstream, with the standard libraries
          # for Darwin that Nixpkgs cannot build on GNU/Linux
          rustBin = (inputs.rust-overlay.lib.mkRustBin { } pkgs).stable.${pkgs.rustc.version};
          targets = [
            "aarch64-apple-darwin"
            "x86_64-apple-darwin"
          ];

          # Apple's SDK as shipped, since the apple-sdk package only builds on
          # Darwin, linked by the lld that comes with the toolchain
          cross = {
            SDKROOT = "${pkgs.apple-sdk_26.src}";
            CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER = "rust-lld";
            CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER = "rust-lld";
          };

          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./.cargo
              ./Cargo.toml
              ./Cargo.lock
              ./src
              ./xtask
            ];
          };

          meta = {
            inherit (crate) description;
            license = lib.licenses.gpl3Plus;
            mainProgram = crate.name;
          };
        in
        {
          packages =
            lib.optionalAttrs isDarwin {
              default = config.packages.scrollreversed;

              scrollreversed = pkgs.rustPlatform.buildRustPackage {
                pname = crate.name;
                inherit (crate) version;
                inherit src meta;
                cargoLock.lockFile = ./Cargo.lock;
                nativeBuildInputs = with pkgs; [ installShellFiles ];
                env.SOURCE_DATE_EPOCH = sourceDateEpoch;
                postInstall = ''
                  cargo xtask man
                  installManPage target/man/scrollreversed.{1,5}
                '';
              };
            }
            // lib.optionalAttrs isLinux {
              default = config.packages.aarch64-darwin-cross;

              aarch64-darwin-cross = pkgs.stdenv.mkDerivation {
                pname = "${crate.name}-aarch64-darwin";
                inherit (crate) version;
                inherit src meta;
                cargoDeps = pkgs.rustPlatform.importCargoLock { lockFile = ./Cargo.lock; };
                nativeBuildInputs = [
                  (rustBin.minimal.override { targets = [ "aarch64-apple-darwin" ]; })
                ]
                ++ (with pkgs; [
                  rustPlatform.cargoSetupHook
                  installShellFiles
                ]);
                env = cross // {
                  SOURCE_DATE_EPOCH = sourceDateEpoch;
                };
                # Cargo strips the Mach-O binary, which GNU strip cannot read
                dontStrip = true;

                buildPhase = ''
                  runHook preBuild
                  cargo build --frozen --release --target aarch64-apple-darwin
                  cargo xtask man
                  runHook postBuild
                '';

                # Only the platform-independent tests can run on the build host
                doCheck = true;
                checkPhase = ''
                  runHook preCheck
                  cargo test --frozen --workspace
                  runHook postCheck
                '';

                installPhase = ''
                  runHook preInstall
                  install -Dm755 -t "$out/bin" target/aarch64-apple-darwin/release/${crate.name}
                  installManPage target/man/scrollreversed.{1,5}
                  runHook postInstall
                '';
              };
            };

          devShells.default = pkgs.mkShell {
            packages = [
              rustfmt
              (rustBin.default.override {
                inherit targets;
                extensions = [
                  "rust-analyzer"
                  "rust-src"
                ];
              })
            ]
            ++ (with pkgs; [ reuse ]);
            env = {
              RUSTFMT = lib.getExe rustfmt;
            }
            // lib.optionalAttrs isLinux cross;
          };

          checks = {
            inherit (config.packages) default;
            reuse = pkgs.runCommandLocal "reuse-lint" { nativeBuildInputs = with pkgs; [ reuse ]; } ''
              reuse --root ${./.} lint
              touch "$out"
            '';
          };

          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              nixfmt.enable = true;
              rumdl-format.enable = true;
              rustfmt = {
                enable = true;
                package = rustfmt;
                inherit (crate) edition;
              };
              taplo.enable = true;
              xmllint.enable = true;
            };
            settings.formatter.xmllint.includes = [ "*.plist" ];
          };
        };
    };
}
