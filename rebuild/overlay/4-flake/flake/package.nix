# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

{
  perSystem =
    { config, pkgs, ... }:
    {
      packages = {
        default = config.packages.rv64_cpu;
        rv64_cpu = pkgs.callPackage ./rv64_cpu.nix { };
      };
    };
}
