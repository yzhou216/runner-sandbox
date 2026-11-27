# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

/*
  The galago configuration of the flake, with the newest stable kernel
  in place of linuxPackages_testing.  At the tested commit, 7.4-rc4
  does not build: the xhci patch of the flake is already upstream, and
  the bcachefs module of nixpkgs fails to compile against 7.4.  The
  console moves to the serial port, where the test answers the
  passphrase prompt.  Neither change touches the disk layout, and
  disko/bcachefs-encrypted.nix is used as it is.

  `stock` is the same system with the initrd unlock units of NixOS as
  they are, one for each of /, /nix, and /home, to count the prompts
  that the flake's change saves.
*/
{ flake }:
let
  galago = (builtins.getFlake flake).nixosConfigurations.galago.extendModules {
    modules = [
      (
        { lib, pkgs, ... }:
        {
          boot.kernelPackages = lib.mkForce pkgs.linuxPackages_latest;
          boot.kernelPatches = lib.mkForce [ ];
          boot.kernelParams = [ "console=ttyS0,115200n8" ];
        }
      )
    ];
  };

  stock = galago.extendModules {
    modules = [
      (
        { lib, ... }:
        {
          boot.initrd.systemd.services = {
            unlock-bcachefs-nix.requiredBy = lib.mkOverride 40 [ "sysroot-nix.mount" ];
            unlock-bcachefs-home.requiredBy = lib.mkOverride 40 [ "sysroot-home.mount" ];
          };
        }
      )
    ];
  };
in
{
  inherit (galago.config.system.build) toplevel diskoScript;
  stock = stock.config.system.build.toplevel;
}
