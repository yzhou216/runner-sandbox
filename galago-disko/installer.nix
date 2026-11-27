# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

/*
  The bcachefs-iso installer of the flake, made reachable over SSH for
  nixos-anywhere.  None of these changes touch the disk layout under
  test.
*/
{ flake, pubKey }:
let
  installer = (builtins.getFlake flake).nixosConfigurations.bcachefs-iso.extendModules {
    modules = [
      (
        { lib, ... }:
        {
          # Headless runner, and the desktop pulls in builds not yet cached
          services.desktopManager.cosmic.enable = lib.mkForce false;
          services.displayManager.cosmic-greeter.enable = lib.mkForce false;
          programs.firefox.enable = lib.mkForce false;

          users.users.root.openssh.authorizedKeys.keys = [ pubKey ];
          boot.kernelParams = [ "console=ttyS0,115200n8" ];
        }
      )
    ];
  };
in
installer.config.system.build.isoImage
