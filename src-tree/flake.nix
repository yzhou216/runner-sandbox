# SPDX-FileCopyrightText: 2025-2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

{
  description = "Write and compile sheet music with LilyPond";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=master";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, nixpkgs, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.platforms.all;

      perSystem =
        { system, pkgs, ... }:
        with pkgs;
        let
          fontconfigFile = makeFontsConf {
            fontDirectories = [ gyre-fonts ];
          };
          # Just enough LaTeX for src/manuscript, kept explicit so the
          # sandboxed build stays offline
          texlive = texliveBasic.withPackages (
            ps: with ps; [
              geometry
              hyperref
              pgf
              tex-gyre
              xcolor
            ]
          );
        in
        {
          _module.args.pkgs = import nixpkgs { inherit system; };

          devShells.default = mkShell {
            packages = [
              gnumake
              lilypond-unstable-with-fonts
              python3Packages.python-ly
              reuse
              texlive
              wildmidi
            ];

            env.FONTCONFIG_FILE = fontconfigFile;
          };

          # Every file carries an SPDX header naming its copyright and
          # license, and every license so named has a text under LICENSES
          checks.reuse = runCommand "check-reuse" { nativeBuildInputs = [ reuse ]; } ''
            reuse --root ${./.} lint
            touch "$out"
          '';

          packages.default = stdenvNoCC.mkDerivation {
            name = "scores";
            src = ./.;

            strictDeps = true;

            nativeBuildInputs = [
              lilypond-unstable-with-fonts
              texlive
            ];

            env.FONTCONFIG_FILE = fontconfigFile;

            preBuild = ''
              export XDG_CACHE_HOME="$(mktemp -d)"
              export TEXMFVAR="$(mktemp -d)"
            '';

            installPhase = ''
              runHook preInstall

              mkdir "$out"
              cp --recursive dist "$_"

              runHook postInstall
            '';
          };
        };
    };
}
