# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

{
  perSystem =
    { pkgs, riscv64, ... }:
    {
      # Shell in which make assembles the test programs with the RISC-V
      # binutils, which AS and OBJDUMP name
      devShells.default = riscv64.mkShell {
        packages = with pkgs; [
          digital
          python3
          reuse
        ];
      };
    };
}
