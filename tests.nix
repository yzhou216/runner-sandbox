# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Test programs of cinq for Windows, built as its cross builds are
let
  flake = builtins.getFlake (toString ./cinq);
  tests =
    name: target:
    flake.packages.x86_64-linux.${name}.overrideAttrs {
      pname = "cinq-tests-${target}";
      buildPhase = "cargo test --frozen --release --no-run --target ${target}";
      installPhase = ''
        mkdir -p "$out"
        cp target/${target}/release/deps/cinq-*.exe "$out"
      '';
    };
in
{
  x86_64 = tests "x86_64-windows-cross" "x86_64-pc-windows-gnullvm";
  aarch64 = tests "aarch64-windows-cross" "aarch64-pc-windows-gnullvm";
}
