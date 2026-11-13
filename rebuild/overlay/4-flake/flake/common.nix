# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

{
  perSystem =
    { pkgs, ... }:
    {
      # Nixpkgs that cross-builds for riscv64 GNU/Linux, whose binutils
      # assemble the test programs
      _module.args.riscv64 = pkgs.pkgsCross.riscv64;
    };
}
