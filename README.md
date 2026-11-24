<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# The GHC bootstrap chain on riscv64, built by Nix

`.github/workflows/riscv64-nix.yml` builds the chain of
[nixpkgs#591060](https://github.com/NixOS/nixpkgs/pull/591060) (Hugs ->
MicroHs -> GHC 6.2.2) with the riscv64 support of the yzhou216/nixpkgs branch
`ghc-bootstrap-503150-riscv64` (commit 4ce9145b), by Nix itself on a RISE
`ubuntu-24.04-riscv` runner.  cache.nixos.org has next to nothing for
riscv64-linux, so Nix first builds the stdenv from nixpkgs's riscv64
bootstrap tools.  After each long stage, every store path goes into a file
binary cache that actions/cache keeps, and the next run resumes from it.

## Outcome

The stdenv was not finished and the chain was not reached.  Five of six long
jobs lost their runner ("The self-hosted runner lost communication with the
server"), and a lost job leaves no log.

- Run 45089298796, cancelled to split the build into steps: the board
  (kernel 5.10.113-scw1, Sv39, 4 cores, 15 GiB, 37 GB free) has the address
  layout that 6.2.2's fixed heap address needs; with an unlimited stack the
  libraries map at 0x1556123000, above the heap window's end at 0x1500000000.
- Run 45089536119: every C compile of the stdenv segfaults.  The bootstrap
  tools' gcc, g++, cpp, collect2 and lto-wrapper are non-PIE executables at
  0x10000; unpacking them, patchelf grows their RPATH and prepends a LOAD at
  0xf000, below the runner's `vm.mmap_min_addr` of 65536, so they run only
  as root.  This is
  [riseproject-dev/riscv-runner#138](https://github.com/riseproject-dev/riscv-runner/issues/138),
  fixed on the Nixpkgs side by the open
  [NixOS/nixpkgs#514892](https://github.com/NixOS/nixpkgs/pull/514892).
  `.github/workflows/bootstrap-tools-probe.yml` (run 45090967386) shows it:
  the compile fails as the runner user, inside and outside the sandbox, and
  succeeds as root outside it.  patchelf 0.15.0, 0.18.0 and 0.19.2 all place
  the new LOAD below 0x10000.
- Run 45091613851: with the stdenv built as root outside the sandbox, the
  bootstrap compiles run; the runner was lost 37 minutes in.
- Run 45105648117: the runner was lost during the nixpkgs checkout.
- Run 45107322560: the stdenv's xgcc stage built in 8 h 37 min and was saved;
  the runner was lost 6 to 8 hours into the rest of the stdenv.
- Run 45263033897, attempt 1: the glibc stage built in 1 h 43 min and was
  saved; the runner was lost about 4 hours into the final gcc.  Attempt 2
  lost its runner about 10 minutes into the final gcc.

The cache entry `riscv64-nix-45263033897-2-glibc` holds the xgcc and glibc
stages.
