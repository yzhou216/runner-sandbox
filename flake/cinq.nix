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
  # ALSA, which cpal plays sound through on GNU/Linux and FreeBSD, and
  # which on FreeBSD takes the monotonic clock in place of Linux's raw
  # one, as FreeBSD's own port of it does, until NixOS/nixpkgs#584781
  # does so in Nixpkgs
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
  # winit and wgpu for Wayland alone, and libdbus, through which rfd
  # reaches the file chooser portal
  runtimeLibs = lib.optionals isLinux [
    dbus
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

  buildInputs = lib.optionals (isLinux || isFreeBSD) [ alsa ];

  env.SOURCE_DATE_EPOCH = sourceDateEpoch;

  # Share directory that the build lays out, whose desktop entry is
  # checked against its specification, and on Darwin the app bundle
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
