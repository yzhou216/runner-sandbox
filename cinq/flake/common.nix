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
        # Ad hoc signature of the whole bundle, under the identifier of its
        # Info.plist, once the fixup has signed the program alone under the
        # name of the file that rustc wrote.  Darwin keeps the operator's
        # answer on reaching the local network under the one and looks the
        # program up by the other, so that light cues stayed off the network
        # after the operator allowed them.
        signBundle = ''
          ${pkgs.rcodesign}/bin/rcodesign sign "$out/Applications/cinq.app"
        '';
      };
    };
}
