# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

# Tutorial in PDF, with screenshots that the build takes of `cinq` playing
# `demo` in a headless sway session, which runs on GNU/Linux alone, and the
# demo beside it, which the tutorial has its readers copy.  Packages for
# other systems can take both from a build of this on GNU/Linux, such as in
# a virtual machine.
{
  lib,
  stdenvNoCC,
  makeFontsConf,
  alsa-plugins,
  grim,
  liberation_ttf,
  oxipng,
  pulseaudio,
  sway-unwrapped,
  texliveBasic,
  wtype,
  cinq,
  demo,
  crate,
  sourceDateEpoch,
}:
let
  root = ../doc/tutorial;
in
stdenvNoCC.mkDerivation {
  pname = "${crate.name}-doc";
  inherit (crate) version;

  # Sources alone, so that what TeX leaves beside them in a working tree
  # never reaches the build
  src = lib.fileset.toSource {
    inherit root;
    fileset = lib.fileset.fileFilter (
      file:
      lib.any file.hasExt [
        "sh"
        "tex"
      ]
    ) root;
  };

  nativeBuildInputs = [
    grim
    oxipng
    pulseaudio
    sway-unwrapped
    (texliveBasic.withPackages (ps: [ ps.float ]))
    wtype
  ];

  env = {
    ALSA_PLUGIN_DIR = "${alsa-plugins}/lib/alsa-lib";
    # Font that cinq draws in, which the sandbox lacks otherwise
    FONTCONFIG_FILE = makeFontsConf { fontDirectories = [ liberation_ttf ]; };
    # Date of the latest commit on the title page as well as in the PDF's
    # own dates
    SOURCE_DATE_EPOCH = sourceDateEpoch;
    FORCE_SOURCE_DATE = "1";
  };

  # Each run of pdflatex reads the contents and the figure numbers that the
  # run before it wrote, which the contents themselves move, so runs go on
  # until they settle.  A reference left undefined fails the build rather
  # than printing as question marks.
  buildPhase = ''
    runHook preBuild
    cp -R ${demo} show
    chmod -R u+w show
    if ! bash shots.sh ${lib.getExe cinq} show screenshots; then
      tail -n 50 screenshots/*.log
      exit 1
    fi
    oxipng --quiet --strip safe screenshots/*.png
    for _ in 1 2 3 4; do
      pdflatex -interaction=nonstopmode -halt-on-error tutorial.tex
      grep -q 'Rerun to get' tutorial.log || break
    done
    if grep -E 'Rerun to get|undefined references' tutorial.log; then
      exit 1
    fi
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    install -Dm644 -t "$out/share/doc/${crate.name}" tutorial.pdf
    cp -R --no-preserve=mode ${demo} "$out/share/doc/${crate.name}/demo"
    runHook postInstall
  '';

  meta = {
    description = "Tutorial of ${crate.name}";
    license = lib.licenses.gpl3Plus;
    platforms = lib.platforms.linux;
  };
}
