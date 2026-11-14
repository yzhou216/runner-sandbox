# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# NixOS virtual machine with the kernel's loopback card, in which rt.sh
# plays cinq of the tree in cinq/, whose thread of sound keeps its usual
# scheduling, and of the tree in cinq-rt/, built with cpal's feature
# realtime, which asks for real-time scheduling, on busy processors.  The
# flakes of cinq come from the clone at WORKSPACE as a Git repository.
#
# Usage: nix build --impure --expr 'import ./vm-rt.nix { workspace = "WORKSPACE"; }'
{ workspace }:
let
  flake = dir: builtins.getFlake "git+file://${workspace}?dir=${dir}&shallow=1";
  system = "x86_64-linux";
  pkgs = (flake "cinq").inputs.nixpkgs.legacyPackages.${system};
  usual = (flake "cinq").packages.${system}.cinq;
  realtime = (flake "cinq-rt").packages.${system}.cinq;
in
pkgs.testers.runNixOSTest {
  name = "cinq-realtime";
  nodes.machine =
    { pkgs, ... }:
    {
      boot.kernelModules = [ "snd-aloop" ];
      boot.extraModprobeConfig = "options snd-aloop pcm_substreams=1";
      environment.systemPackages = with pkgs; [
        sway
        wtype
        grim
        alsa-utils
        ffmpeg
        procps
      ];
      virtualisation.memorySize = 4096;
      virtualisation.cores = 2;
    };
  testScript = ''
    machine.wait_for_unit("multi-user.target")
    machine.succeed("${pkgs.bash}/bin/bash ${./rt.sh} /tmp/rt usual=${usual}/bin/cinq realtime=${realtime}/bin/cinq >&2")
    machine.copy_from_machine("/tmp/rt", "")
  '';
}
