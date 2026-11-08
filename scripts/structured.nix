# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Smallest derivation for x86_64-freebsd, built by the bash of FreeBSD's
# packages with structured attributes or without them, which tells Nix's
# part in an abort from that of Nixpkgs' bootstrap tools
{ structured }:
builtins.derivation (
  {
    name = if structured then "structured" else "plain";
    system = "x86_64-freebsd";
    builder = "/usr/local/bin/bash";
    args = [
      "-c"
      (
        if structured then
          ''source "$NIX_ATTRS_SH_FILE"; echo hello > "''${outputs[out]}"''
        else
          ''echo hello > "$out"''
      )
    ];
  }
  // (if structured then { __structuredAttrs = true; } else { })
)
