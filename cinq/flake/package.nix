# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

{
  perSystem =
    {
      config,
      pkgs,
      system,
      crate,
      sourceDateEpoch,
      installBundle,
      ...
    }:
    {
      packages = {
        default = config.packages.cinq;
        # Named after the system that it runs on, as the builds for other
        # systems are, so that each system's name gives cinq for it
        ${system} = config.packages.cinq;

        cinq = pkgs.callPackage ./cinq.nix { inherit crate sourceDateEpoch installBundle; };
      };
    };
}
