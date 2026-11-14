# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# NixOS virtual machine with the kernel's loopback card, which the kernels
# of the runners lack, in which alsa.sh plays cinq of the tree in cinq/ on
# ALSA's hw devices, held to the quirks of sound cards
{
  inputs.cinq.url = "path:./cinq";

  outputs =
    { cinq, ... }:
    let
      system = "x86_64-linux";
      pkgs = cinq.inputs.nixpkgs.legacyPackages.${system};
      build = cinq.packages.${system}.cinq;
    in
    {
      packages.${system}.alsa = pkgs.testers.runNixOSTest {
        name = "cinq-alsa-quirks";
        nodes.machine =
          { pkgs, ... }:
          {
            boot.kernelModules = [ "snd-aloop" ];
            boot.extraModprobeConfig = "options snd-aloop pcm_substreams=1";
            environment.systemPackages =
              (with pkgs; [
                sway
                wtype
                grim
                alsa-utils
                ffmpeg
              ])
              ++ [ build ];
            virtualisation.memorySize = 4096;
            virtualisation.cores = 2;
          };
        testScript = ''
          machine.wait_for_unit("multi-user.target")
          machine.succeed("cat /proc/asound/cards >&2")
          machine.succeed("${pkgs.bash}/bin/bash ${./alsa.sh} ${build}/bin/cinq /tmp/alsa >&2")
          machine.copy_from_vm("/tmp/alsa", "")
        '';
      };
    };
}
