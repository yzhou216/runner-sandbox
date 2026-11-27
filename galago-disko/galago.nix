# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

/*
  The galago configuration of the flake, with the newest stable kernel
  in place of linuxPackages_testing.  At the tested commit, 7.4-rc4
  does not build: the xhci patch of the flake is already upstream, and
  the bcachefs module of nixpkgs fails to compile against 7.4.  The
  kernel does not change the disk layout, and disko/bcachefs.nix is
  used as it is.
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
        }
      )
    ];
  };
in
{
  inherit (galago.config.system.build) toplevel diskoScript;
}
