# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

{ inputs, ... }:
{
  perSystem =
    { lib, pkgs, ... }:
    {
      # Values that the other files share, which their perSystem functions
      # take as arguments, just as they take pkgs
      _module.args = {
        crate = (lib.importTOML ../Cargo.toml).package;
        # Date of the latest commit, which the manual pages and packages
        # give rather than the fixed date that builds take otherwise
        sourceDateEpoch = toString inputs.self.lastModified;
        # Rustfmt with the unstable settings of rustfmt.toml turned on
        rustfmt = pkgs.rustfmt.override { asNightly = true; };
        # Moves the program into the app bundle that builds for Darwin lay
        # out, and links it back from bin
        installBundle = ''
          cp -r target/Applications "$out"
          mv "$out/bin/cinq" "$out/Applications/cinq.app/Contents/MacOS"
          ln -s ../Applications/cinq.app/Contents/MacOS/cinq "$out/bin"
        '';
        # Libraries that cinq loads at run time rather than links: those
        # of winit and wgpu for Wayland alone, and libdbus, through which
        # rfd reaches the file chooser portal
        runtimeLibs = lib.optionals pkgs.stdenvNoCC.hostPlatform.isLinux (
          with pkgs;
          [
            dbus
            libGL
            libxkbcommon
            vulkan-loader
            wayland
          ]
        );
      };
    };
}
