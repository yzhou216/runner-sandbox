# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

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
      runtimeLibs,
      ...
    }:
    let
      inherit (pkgs.stdenvNoCC.hostPlatform) isDarwin isFreeBSD isLinux;
    in
    {
      packages = {
        default = config.packages.cinq;
        # Named after the system that it runs on, as the builds for other
        # systems are, so that each system's name gives cinq for it
        ${system} = config.packages.cinq;

        cinq = pkgs.rustPlatform.buildRustPackage {
          pname = crate.name;
          inherit (crate) version;

          src = lib.fileset.toSource {
            root = ../.;
            fileset = lib.fileset.unions [
              ../.cargo
              ../Cargo.toml
              ../Cargo.lock
              ../build.rs
              ../src
              ../man
              ../share
            ];
          };

          cargoLock.lockFile = ../Cargo.lock;

          nativeBuildInputs = with pkgs; [
            pkg-config
            scdoc
            desktop-file-utils
          ];

          # ALSA, which cpal plays sound through on GNU/Linux and FreeBSD
          buildInputs = lib.optionals (isLinux || isFreeBSD) [ pkgs.alsa-lib ];

          env.SOURCE_DATE_EPOCH = sourceDateEpoch;

          # Share directory that the build lays out, whose desktop entry
          # is checked against its specification, and on Darwin the app
          # bundle
          postInstall = ''
            cp -r target/share "$out"
            desktop-file-validate "$out/share/applications/cinq.desktop"
          ''
          + lib.optionalString isDarwin installBundle;

          postFixup = lib.optionalString isLinux ''
            patchelf --add-rpath ${lib.makeLibraryPath runtimeLibs} "$out/bin/cinq"
          '';

          meta = {
            inherit (crate) description;
            license = lib.licenses.gpl3Plus;
            mainProgram = "cinq";
          };
        };
      };
    };
}
