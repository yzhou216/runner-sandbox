# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

{
  perSystem =
    {
      config,
      lib,
      pkgs,
      crate,
      sourceDateEpoch,
      ...
    }:
    let
      # glibc of Nixpkgs, which cinq links against, so that other
      # distributions need this one or a newer one
      glibc = lib.versions.majorMinor pkgs.glibc.version;
      # cinq for other distributions of GNU/Linux, with their loader and
      # libraries in place of the Nix store's
      portable =
        let
          directory = if pkgs.stdenv.hostPlatform.isx86_64 then "lib64" else "lib";
          loader = "/${directory}/${baseNameOf pkgs.stdenv.cc.bintools.dynamicLinker}";
        in
        pkgs.runCommand "${crate.name}-portable" { nativeBuildInputs = [ pkgs.patchelf ]; } ''
          mkdir -p "$out/usr"
          cp -r ${config.packages.cinq}/{bin,share} "$out/usr"
          chmod -R u+w "$out/usr"
          patchelf --set-interpreter ${loader} --remove-rpath "$out/usr/bin/cinq"
        '';
      # Package that depends on the unwinder of GCC, which Rust's standard
      # library takes, but on no library of the GPU, without which cinq
      # draws in software
      nfpm =
        packager:
        pkgs.runCommand "${crate.name}-${packager}"
          {
            nativeBuildInputs = [ pkgs.nfpm ];
            env.SOURCE_DATE_EPOCH = sourceDateEpoch;
            nfpmConfig = builtins.toJSON rec {
              inherit (crate)
                name
                version
                description
                license
                ;
              arch = pkgs.stdenv.hostPlatform.go.GOARCH;
              maintainer = "Yiyu Zhou <yiyu@yiyuzhou.io>";
              section = "sound";
              archlinux.packager = maintainer;
              # Files alone, leaving their directories to the system
              contents = [
                {
                  src = "usr/";
                  dst = "/usr";
                }
              ];
              overrides = {
                deb.depends = [
                  "libc6 (>= ${glibc})"
                  "libasound2t64 | libasound2"
                  "libgcc-s1"
                  "libdbus-1-3"
                  "libwayland-client0"
                  "libxkbcommon0"
                ];
                rpm.depends = [
                  "glibc >= ${glibc}"
                  "alsa-lib"
                  "libgcc"
                  "dbus-libs"
                  "libwayland-client"
                  "libxkbcommon"
                ];
                archlinux.depends = [
                  "glibc"
                  "alsa-lib"
                  "gcc-libs"
                  "dbus"
                  "wayland"
                  "libxkbcommon"
                ];
              };
            };
            passAsFile = [ "nfpmConfig" ];
          }
          ''
            cp -r ${portable}/usr usr
            chmod -R u+w usr
            mkdir "$out"
            nfpm package --config "$nfpmConfigPath" --packager ${packager} --target "$out"
          '';
    in
    {
      packages = lib.optionalAttrs pkgs.stdenvNoCC.hostPlatform.isLinux {
        deb = nfpm "deb";
        rpm = nfpm "rpm";
        archlinux = nfpm "archlinux";
      };
    };
}
