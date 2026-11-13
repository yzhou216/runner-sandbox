# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

{ inputs, ... }:
{
  imports = [ inputs.treefmt-nix.flakeModule ];

  perSystem =
    { pkgs, riscv64, ... }:
    {
      checks = {
        reuse = pkgs.runCommandLocal "reuse-lint" { nativeBuildInputs = [ pkgs.reuse ]; } ''
          reuse --root ${../.} lint
          touch "$out"
        '';
        roms =
          pkgs.runCommandLocal "rv64_cpu-roms"
            {
              nativeBuildInputs = [
                pkgs.gnumake
                pkgs.python3
              ];
            }
            ''
              python3 ${./roms.py} ${../.} \
                ${riscv64.buildPackages.binutils}/bin/riscv64-unknown-linux-gnu-{as,objdump}
              touch "$out"
            '';
        programs = pkgs.runCommandLocal "rv64_cpu-programs" { } ''
          sh ${./programs.sh} ${pkgs.jre}/bin/java ${pkgs.digital}/share/java/Digital.jar ${../src}
          touch "$out"
        '';
      };

      treefmt = {
        projectRootFile = "flake.nix";
        programs = {
          nixfmt.enable = true;
          rumdl-format.enable = true;
          shfmt = {
            enable = true;
            indent_size = 0; # Tabs
          };
          taplo.enable = true;
          yamlfmt = {
            enable = true;
            settings.formatter.retain_line_breaks_single = true; # Blank line after the license
          };
        };
      };
    };
}
