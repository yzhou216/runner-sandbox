# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

{ inputs, ... }:
{
  imports = [ inputs.treefmt-nix.flakeModule ];

  perSystem =
    {
      config,
      lib,
      pkgs,
      system,
      crate,
      rustfmt,
      ...
    }:
    {
      checks = {
        cinq = config.packages.cinq; # Its build runs the tests
        reuse = pkgs.runCommandLocal "reuse-lint" { nativeBuildInputs = [ pkgs.reuse ]; } ''
          reuse --root ${../.} lint
          touch "$out"
        '';
      }
      # Tutorial where this system builds it itself, rather than a builder
      // lib.optionalAttrs (config.packages.doc.system == system) { inherit (config.packages) doc; };

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
          yamlfmt = {
            enable = true;
            settings.formatter.retain_line_breaks_single = true; # Blank line after the license
          };
        };
      };
    };
}
