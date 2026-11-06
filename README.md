<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# runner-sandbox: RISE RISC-V runner

Probed `ubuntu-24.04-riscv`, which the RISE RISC-V Runners Personal GitHub
App serves to this repository, and installed Nix on it.

Outcome, in run 42179610929: the job passed.

- The job queued for 2 minutes 8 seconds before runner
  `rise-riscv-runner-ke6ez8rnn` took it, then ran for 1 minute 54 seconds.
- The runner was a Docker container, with `docker-init` as PID 1, on Linux
  5.10.113-scw1 and Ubuntu 24.04.5.  It had 4 CPUs with ISA `rv64imafdcsu`,
  so no vector extension, 15 GiB of memory, no swap, and 76 GB free.  The
  job ran as `runner`, uid 1001, with `sudo`.
- `gcc`, `python3`, `go`, `rustc`, `cargo`, `java`, `docker` 28.0.4, and
  `podman` were present, but `clang` and `node` were not on `PATH`, although
  RISE's documentation lists them.  `actions/checkout` ran regardless.
- `cachix/install-nix-action@v31` installed Nix 2.35.4 in 16 seconds, and a
  sandboxed build of a trivial derivation worked.  `nix build nixpkgs#hello
  --dry-run` took 80 seconds and listed 133 derivations to build and 67 paths
  to fetch, so cache.nixos.org lacks most of stdenv for riscv64-linux.
- `actionlint` rejected the label as unknown until `.github/actionlint.yaml`
  listed it under `self-hosted-runner`.
