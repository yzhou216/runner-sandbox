# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

{
  perSystem =
    {
      config,
      lib,
      pkgs,
      rustfmt,
      runtimeLibs,
      ...
    }:
    {
      devShells.default = pkgs.mkShell {
        inputsFrom = [ config.packages.cinq ];

        packages = [
          rustfmt
        ]
        ++ (with pkgs; [
          clippy
          reuse
          rust-analyzer
        ]);

        env = lib.optionalAttrs pkgs.stdenvNoCC.hostPlatform.isLinux {
          LD_LIBRARY_PATH = lib.makeLibraryPath runtimeLibs;
        };
      };
    };
}
