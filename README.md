<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: builds for riscv64 GNU/Linux and FreeBSD

Tried every build of cinq that its CI was to gain, on the snapshot of cinq
in `cinq/`: riscv64 GNU/Linux and FreeBSD natively and cross-built, then
whether Nix builds every system that cinq supports, cross-built on x86_64
and on aarch64 GNU/Linux, within the six hours of a job on GitHub's
runners, with no cache but cache.nixos.org.  Branch
`cinq/2026-11-08-ci-as-committed` ran cinq's own workflows as committed at
ff05475, and all 32 jobs passed (runs 42430388405, 42430388414, and
42430388475).

## Native builds with cargo

`builds.yml`, runs 42405137740 and 42406449705.

- riscv64 on RISE's `ubuntu-24.04-riscv`: 237 tests passed; testing took 50
  to 61 minutes and the build 20 to 25 more from scratch.
- x86_64 FreeBSD 15.1 in `vmactions/freebsd-vm`: one test failed at first,
  since kqueue tells a directory's watcher nothing of a file in it written
  in place; with cinq watching `config.toml` itself on kqueue, all 237
  passed, in about 10 minutes.
- Programs cross-built by Nix on x86_64-linux started on riscv64 and
  FreeBSD.

## FreeBSD on aarch64 and riscv64

`freebsd-arches.yml`, run 42410882955.

- GitHub's arm64 runners have no `/dev/kvm`, so FreeBSD for aarch64 runs
  emulated there as on x86_64.
- The riscv64 image's package repository has no Rust, and Nixpkgs refuses
  riscv64-freebsd.
- Nixpkgs cross-built cinq for aarch64-freebsd in 22 minutes, Rust's
  compiler included, and it started in an emulated machine.

## Nix for every system

`nix-everywhere.yml`, runs 42412315915, 42423036697, 42426347873, and
42427246421 (the last, with every fix, passed all 29 jobs).

- All 14 cross builds, from x86_64-linux and aarch64-linux to the other
  systems of x86_64-linux, aarch64-linux, riscv64-linux, aarch64-darwin,
  x86_64-windows, aarch64-windows, x86_64-freebsd, and aarch64-freebsd,
  took 3 to 30 minutes, and each started on its own system.
- aarch64-linux to x86_64-freebsd needed a fix: Nixpkgs' clang cpp
  preprocesses for the build platform, which on aarch64 rejected FreeBSD's
  x86 flags and wrote empty symbol maps, so libthr exported 7 symbols in
  place of 312 (`freebsd-libc.yml`, runs 42424765022, 42425333973,
  42425472510, and 42425674224).  A script that gives the cpp FreeBSD's
  target fixes it.
- Native Nix on aarch64-linux: `nix flake check` and `nix build` in 7
  minutes.

## Native Nix on riscv64 and FreeBSD

- RISE (`riscv64-nix.yml`, runs 42422671372 and 42424765023): the `gcc` of
  Nixpkgs' riscv64 bootstrap tools dies with SIGSEGV in the ELF loader of
  the board's kernel, 5.10.113-scw1, before its first instruction, inside
  Nix or out.  `ubuntu-26.04-riscv` got no runner (run 42423036804).
- FreeBSD (`freebsd-nix.yml`, runs 42422671348, 42423036828, and
  42424765105): Nix 2.32.4 from FreeBSD's packages needs `NIX_CONFIG`, a CA
  file, and a workspace that root owns; then it plans 1565 derivations,
  and every builder with structured attributes aborts with exit code 134.
