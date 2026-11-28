<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq line-cutting round on GNU/Linux, eight checks, OLD and NEW

Branch `cinq/2026-11-27-linux-loc`, a rerun of `cinq/2026-11-24-linux-final` with a matrix over the build.  OLD is
`e304bd4e581af53fa32c5ae9a598c2d655130e61` (master), NEW is `efa362c0f5193bc3bd73bbc4494284fa6c6429ce`.  Run
**46111216048**, green first time on `ubuntu-latest` (x86_64) and `ubuntu-24.04-arm` (aarch64), no rerun.  The script
never fails on a bad check, so `steps.txt` was diffed between OLD and NEW per arch and screenshots were read.

**No difference between OLD and NEW.**  Diffs of `steps.txt` are only temporary directory names.  Screenshots that are
not byte-identical differ in timers (`0:04.6 left` against `0:04.5 left`) and were checked by eye on x86_64.  Stderr
output is identical in words.

| # | Check | x86_64 OLD | x86_64 NEW | aarch64 OLD | aarch64 NEW |
| --- | --- | --- | --- | --- | --- |
| 1 | Silent start | pass | pass | pass | pass |
| 2 | `keymap = "cua"` | pass | pass | pass | pass |
| 3 | Hint mode | pass | pass | pass | pass |
| 4 | OSC (`/go`, cue 99, port held) | pass | pass | pass | pass |
| 5 | File watching | pass | pass | pass | pass |
| 6 | Paste | pass | pass | pass | pass |
| 7 | Errors in words | pass | pass | pass | pass |
| 8 | Closed port | pass | pass | pass | pass |
