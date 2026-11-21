<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# The GHC bootstrap chain on riscv64

The chain of [nixpkgs#591060](https://github.com/NixOS/nixpkgs/pull/591060)
goes Hugs -> MicroHs -> GHC 6.2.2, and from 6.2.2 up the ladder of
[andreabedini/ghc-bootstrap](https://github.com/andreabedini/ghc-bootstrap).
`.github/workflows/riscv64.yml` tries its first links on a RISE
`ubuntu-24.04-riscv` runner (a `scaleway-em-rv1` board, 4 cores, Ubuntu
24.04), with the runner's own toolchain, since cache.nixos.org has nothing for
riscv64-linux, not even the stdenv.

1. Hugs from nixpkgs's source, with its configure flags and C flags, the
   hsc2hs fix of nixpkgs#591107, and config.sub and config.guess replaced as
   nixpkgs does for every package.  nixpkgs's rewrite of `/bin/cp` is left
   out: it changes every file's time, after which Hugs's Makefile reruns
   autoreconf and configure without the flags wherever autoreconf exists.
2. nixpkgs's Hugs install check.
3. MicroHs 0.16.8.0 stage 1 compiled by that Hugs, with nixpkgs's two Hugs
   patches (`patches/`, from nixpkgs, MIT) and the command of its `boot.nix`
   and `stage1.nix`, then a hello-world compiled by it.
4. GHC 6.2.2's configure, with config.sub and config.guess replaced.

## Outcome

Run 44906649374 (35 minutes):

- Hugs builds; only network, X11, HGL, OpenGL, GLUT, OpenAL and ALUT are
  skipped, as on x86_64-linux, and the install check prints `(55,True,True)`.
- Hugs compiles MicroHs stage 1 in 15 minutes, and the result compiles and
  runs a hello-world.
- GHC 6.2.2 stops at once: `Unrecognised platform: riscv64-unknown-linux-gnu`.
  Its configure.ac knows a fixed list of 2004 platforms, and no GHC release
  before 8.8.1 knows riscv64 (8.8.1 as an unknown, unregisterised one).

So Hugs and MicroHs work on riscv64, and the chain stops at its first GHC.
Run 44902750271 failed at `make install` (the `/bin/cp` rewrite above), and run
44904153189 at a hello-world named by module rather than by file.
