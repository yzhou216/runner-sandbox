# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

# Demo shows with the sounds, the picture, and the film that they play,
# which doc/demo/build.sh makes rather than the repository holding them
{
  lib,
  stdenvNoCC,
  dejavu_fonts,
  ffmpeg-headless,
  crate,
}:
stdenvNoCC.mkDerivation {
  pname = "${crate.name}-demo";
  inherit (crate) version;

  src = ../doc/demo;

  nativeBuildInputs = [ ffmpeg-headless ];

  buildPhase = ''
    runHook preBuild
    ./build.sh "$out" ${dejavu_fonts}/share/fonts/truetype/DejaVuSans.ttf
    runHook postBuild
  '';

  dontInstall = true;

  meta = {
    description = "Demo shows of ${crate.name}";
    license = lib.licenses.gpl3Plus;
  };
}
