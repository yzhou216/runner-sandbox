# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

# cinq for the system of the Nixpkgs whose callPackage calls this, so that
# the Nixpkgs of another system cross-builds the same package
{
  lib,
  stdenvNoCC,
  rustPlatform,
  pkg-config,
  scdoc,
  desktop-file-utils,
  alsa-lib,
  libjack2,
  dbus,
  libGL,
  libxkbcommon,
  vulkan-loader,
  wayland,
  crate,
  sourceDateEpoch,
  installBundle,
}:
let
  inherit (stdenvNoCC.hostPlatform) isDarwin isFreeBSD isLinux;
  # Monotonic clock in place of Linux's raw one, as in FreeBSD's own
  # port of ALSA, until NixOS/nixpkgs#584781 does so in Nixpkgs
  alsa =
    if isFreeBSD then
      alsa-lib.overrideAttrs (old: {
        postPatch = (old.postPatch or "") + ''
          substituteInPlace src/ucm/main.c --replace-fail CLOCK_MONOTONIC_RAW CLOCK_MONOTONIC
        '';
      })
    else
      alsa-lib;
  # Libraries that cinq loads at run time rather than links: those of
  # winit and wgpu for Wayland alone, libdbus, through which rfd reaches the
  # file chooser portal, and libjack, which pw-jack swaps for PipeWire's
  # through LD_LIBRARY_PATH
  runtimeLibs = lib.optionals isLinux [
    dbus
    libjack2
    libGL
    libxkbcommon
    vulkan-loader
    wayland
  ];
in
rustPlatform.buildRustPackage {
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

  nativeBuildInputs = [
    pkg-config
    scdoc
    desktop-file-utils
  ];

  # JACK only for jack-sys, which asks pkg-config for it on GNU/Linux, as
  # cinq loads libjack at run time, and then only where a JACK server runs
  buildInputs = lib.optionals (isLinux || isFreeBSD) [ alsa ] ++ lib.optionals isLinux [ libjack2 ];

  env.SOURCE_DATE_EPOCH = sourceDateEpoch;

  postInstall = ''
    cp -r target/share "$out"
    desktop-file-validate "$out/share/applications/cinq.desktop"
  ''
  + lib.optionalString isDarwin installBundle;

  postFixup = lib.optionalString isLinux ''
    patchelf --add-rpath ${lib.makeLibraryPath runtimeLibs} "$out/bin/cinq"
  '';

  passthru = { inherit runtimeLibs; };

  meta = {
    inherit (crate) description;
    license = lib.licenses.gpl3Plus;
    mainProgram = "cinq";
  };
}
