# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

{ inputs, ... }:
{
  imports = [ inputs.treefmt-nix.flakeModule ];

  perSystem =
    {
      config,
      pkgs,
      crate,
      rustfmt,
      ...
    }:
    {
      # What nix flake check holds the repository to, along with the
      # formatting below: the build of cinq, which runs its tests, and
      # the REUSE specification for every file
      checks = {
        cinq = config.packages.cinq;
        reuse = pkgs.runCommandLocal "reuse-lint" { nativeBuildInputs = [ pkgs.reuse ]; } ''
          reuse --root ${../.} lint
          touch "$out"
        '';
      };

      # Formatters of every kind of file in the repository that has one,
      # which nix fmt runs, and which nix flake check holds the files to
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
