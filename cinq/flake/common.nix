# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

{ inputs, ... }:
{
  perSystem =
    { lib, pkgs, ... }:
    {
      _module.args = {
        crate = (lib.importTOML ../Cargo.toml).package;
        # Date of the latest commit, which the manual pages and packages
        # give rather than the fixed date that builds take otherwise
        sourceDateEpoch = toString inputs.self.lastModified;
        # Rustfmt with the unstable settings of rustfmt.toml turned on
        rustfmt = pkgs.rustfmt.override { asNightly = true; };
        installBundle = ''
          cp -r target/Applications "$out"
          mv "$out/bin/cinq" "$out/Applications/cinq.app/Contents/MacOS"
          ln -s ../Applications/cinq.app/Contents/MacOS/cinq "$out/bin"
        '';
      };
    };
}
