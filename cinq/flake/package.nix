# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

{ inputs, ... }:
{
  perSystem =
    {
      config,
      lib,
      pkgs,
      system,
      crate,
      sourceDateEpoch,
      installBundle,
      ...
    }:
    let
      # Bound here rather than read back from the packages, whose names
      # depend on whether the demo and the tutorial built from them evaluate
      cinq = pkgs.callPackage ./cinq.nix { inherit crate sourceDateEpoch installBundle; };
      demo = pkgs.callPackage ./demo.nix { inherit crate; };
      doc = pkgs.callPackage ./doc.nix {
        inherit
          cinq
          demo
          crate
          sourceDateEpoch
          ;
      };
      evaluates = drv: (builtins.tryEval drv.drvPath).success;
    in
    {
      packages = {
        default = config.packages.cinq;
        # Named after the system that it runs on, as the builds for other
        # systems are, so that each system's name gives cinq for it
        ${system} = config.packages.cinq;

        inherit cinq;
      }
      # Demo wherever ffmpeg evaluates, which it does not on FreeBSD
      // lib.optionalAttrs (evaluates demo) { inherit demo; }
      # Tutorial for every system, built on GNU/Linux, where sway runs: here
      # where it evaluates, else on GNU/Linux of the same processor, or on
      # x86_64-linux for armv6l, armv7l, and powerpc64le, where sway and TeX
      # Live reach rdma-core, whose manual pages take pandoc and so GHC,
      # which Nixpkgs cannot bootstrap there.  Nix builds those on a builder
      # for that system, such as the QEMU virtual machine of nix-darwin's
      # linux-builder.
      // {
        doc =
          if evaluates doc then
            doc
          else
            inputs.self.packages.${
              if pkgs.stdenv.hostPlatform.isLinux then
                "x86_64-linux"
              else
                "${pkgs.stdenv.hostPlatform.parsed.cpu.name}-linux"
            }.doc;
      };
    };
}
