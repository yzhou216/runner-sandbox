# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

# The circuits of rv64_cpu, and a command that opens the processor in
# Digital
{
  lib,
  stdenvNoCC,
  digital,
  makeWrapper,
}:
stdenvNoCC.mkDerivation {
  name = "rv64_cpu";

  src = lib.fileset.toSource {
    root = ../.;
    fileset = ../src;
  };

  nativeBuildInputs = [ makeWrapper ];

  installPhase = ''
    runHook preInstall
    mkdir -p "$out/share"
    cp -r src "$out/share/rv64_cpu"
    makeWrapper ${lib.getExe digital} "$out/bin/rv64_cpu" \
      --add-flags "$out/share/rv64_cpu/rv64_cpu.dig"
    runHook postInstall
  '';

  meta = {
    description = "Single-cycle 64-bit RISC-V processor in Digital";
    license = lib.licenses.gpl3Plus;
    mainProgram = "rv64_cpu";
    inherit (digital.meta) platforms;
  };
}
