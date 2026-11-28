<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq OLD vs NEW after the line-cutting round, on FreeBSD

Rerun of `cinq/2026-11-24-freebsd-final` with a matrix over the build: OLD is
`e304bd4e581af53fa32c5ae9a598c2d655130e61`, NEW is
`efa362c0f5193bc3bd73bbc4494284fa6c6429ce`.  Run **46111331460**, all eight
jobs green first time, no rerun.  Workflow and `fb/check.sh` are unchanged
from the earlier branch except for the matrix.  FreeBSD 15.1, x86_64 native
and aarch64 emulated.

| Check | x86_64 OLD | x86_64 NEW | aarch64 OLD | aarch64 NEW |
| --- | --- | --- | --- | --- |
| 1 Silent start | pass (0 bytes) | pass (0 bytes) | pass | pass |
| 2 Retired keymap | pass | pass | pass | pass |
| 3 Hint mode | pass | pass | pass | pass |
| 4 OSC | pass | pass | pass | pass |
| 5 File watching | pass | pass | pass | pass |
| 6 Paste | pass | pass | pass | pass |
| 7 Errors in words | pass | pass | pass | pass |
| 8 Network cue | pass | pass | pass | pass |

Verdicts were taken from the artifacts, not the job status: standard error
files and saved config.toml files are identical between OLD and NEW apart
from the random temporary directory name in paths; 38 of 56 screenshots are
byte identical, and the rest (countdown seconds in check 4, equal on viewing
in checks 2, 5, 6, 7) differ only in pixel noise or timing.

Difference between OLD and NEW: none.
