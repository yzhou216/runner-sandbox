# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

{
  description = "Geospatial hazard processing";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    inputs@{
      nixpkgs,
      flake-parts,
      crane,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;
      perSystem =
        { pkgs, lib, ... }:
        let
          craneLib = crane.mkLib pkgs;
          # Cargo sources, the Scheme files built into geoai, and the test data
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              (craneLib.fileset.commonCargoSources ./.)
              (lib.fileset.fileFilter (file: file.hasExt "scm") ./.)
              ./tests/data
            ];
          };
          arguments = {
            inherit src;
            strictDeps = true;
          };
          crate = arguments // {
            cargoArtifacts = craneLib.buildDepsOnly arguments;
          };
        in
        {
          packages.default = craneLib.buildPackage (
            crate
            // {
              doCheck = false;
              nativeBuildInputs = [ pkgs.installShellFiles ];
              # Not `cargo xtask`, which would compile geoai again in the dev profile
              postInstall = ''
                cargo run --release --package xtask -- man man
                installManPage man/*
              '';
            }
          );

          checks = {
            cargo-clippy = craneLib.cargoClippy (
              crate // { cargoClippyExtraArgs = "--workspace --all-targets -- --deny warnings"; }
            );

            cargo-test = craneLib.cargoTest crate;

            cargo-fmt = craneLib.cargoFmt { inherit src; };

            rumdl = pkgs.runCommand "rumdl-check" { nativeBuildInputs = [ pkgs.rumdl ]; } ''
              cd ${./.}
              rumdl check --no-cache .
              touch $out
            '';

            reuse = pkgs.runCommand "reuse-lint" { nativeBuildInputs = [ pkgs.reuse ]; } ''
              reuse --root ${./.} lint
              touch $out
            '';
          };

          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              git-lfs
              reuse
              rumdl
            ];
            shellHook = ''
              git config core.hooksPath .githooks
            '';
          };
        };
    };
}
