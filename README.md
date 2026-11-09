<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq's tests on every runner

Runs `cargo test --no-fail-fast` of cinq (copy in `cinq/`, branch
native-tests) on windows-latest, windows-11-arm, macos-latest,
ubuntu-latest, ubuntu-24.04-arm, and FreeBSD 15.1 (vmactions/freebsd-vm).

- 42588158410, unfixed: everything passed except the two known tests on
  both Windows runners (a bare `/show` is not absolute there).  FreeBSD:
  253 passed.
- 42590473447, path fixes: Windows passed (251 tests, two are
  `cfg(unix)`); ubuntu-24.04-arm aborted with a stack overflow in
  `picture_that_the_next_go_starts_is_made_ready_ahead_for_its_run_to_take`.
- 42605787251, repeated runs: that test failed in a few runs of a hundred
  on GNU/Linux and once in 20 on Darwin.  Cause: a race in the test, which
  dropped the picture that the sync after GO handed on.
- 42608766876, race fixed: every runner passed, and no repeated run failed
  (300 and 300 runs, 40 of every test, also on FreeBSD).
