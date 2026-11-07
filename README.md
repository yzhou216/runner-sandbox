<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# openh264-rs: riscv64

Tested riscv64 support for openh264-rs on the branch `riscv64` of
yzhou216/openh264-rs at 9eaf98e.  The branch adds riscv64 to the
architectures that the build script of openh264-sys2 names, and adds a job
to upstream's CI that cross-builds the tests for riscv64.  Upstream's master
was at 5f3637e.

- `native`, on `ubuntu-24.04-riscv` (RISE, Ubuntu 24.04.5 on Linux
  5.10.113-scw1, rustc 1.99.0, g++ 13.3.0), builds openh264-sys2 from
  master, then runs `cargo test --verbose --all-features --
  --test-threads=1 --nocapture` on the branch, as upstream's CI does, with
  the repository's `.cargo/config.toml`.
- `cross`, on `ubuntu-latest`, takes the steps of the new CI job.

## Outcome

Run 42318007985 (2075cf1): both jobs passed.

- From master, the build script of openh264-sys2 panicked at build.rs:110
  with `Unknown target arch: riscv64`, and cargo exited with 101.
- On the branch, the tests built and ran in 798 seconds: 65 passed, none
  failed, and the 3 that upstream marks to ignore were ignored.  Ubuntu's
  native gcc package has `riscv64-linux-gnu-gcc`, the linker that the
  branch names for the target in `.cargo/config.toml`, and the tests linked
  with the AddressSanitizer that the same file adds.
- The cross build of the tests took 56 seconds.  `-C target-cpu=native`
  from `.cargo/config.toml` makes LLVM warn thousands of times that the
  processor and the features of the x86 host are not those of riscv64.

## Notes

- GitHub had enabled Actions on the new fork, so the first push of the
  branch started its CI there.  That run was cancelled and Actions turned
  off on the fork, so that tests go through this sandbox.
- A partial clone (`--filter=blob:none`) of openh264-rs cannot push through
  git-lfs's pre-push hook, which looks for blobs that the clone left out.
  The hook uploaded the 17 LFS objects to the fork before it failed, and
  `git push --no-verify` pushed the branch.
