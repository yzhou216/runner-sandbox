<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq line-cutting round on Windows

Rerun of the eight checks of `cinq/2026-11-24-windows-final` on OLD
`e304bd4e581af53fa32c5ae9a598c2d655130e61` and NEW
`efa362c0f5193bc3bd73bbc4494284fa6c6429ce`, on `windows-latest` and
`windows-11-arm`, both builds in one run (46111210624, green first time, no
rerun).  Only the cinq SHA became a matrix over the build.  The scripts never
fail on a bad check, so the verdicts come from the `checks-*` artifacts:
standard error of every phase compared file by file, and side-by-side
screenshots of OLD and NEW for each check.

| Check | windows-latest OLD / NEW | windows-11-arm OLD / NEW |
| --- | --- | --- |
| 1 Silent start | pass / pass | pass / pass |
| 2 Retired keymap | pass / pass | pass / pass |
| 3 Hint mode | pass / pass | pass / pass |
| 4 OSC | pass / pass | pass / pass |
| 5 File watching | pass / pass | pass / pass |
| 6 Paste | pass / pass | pass / pass |
| 7 Errors in words | pass / pass | pass / pass |
| 8 Closed port | pass / pass | pass / pass |

No difference between OLD and NEW.  Standard error is byte-identical in every
phase except the random temporary file name in check 7b's message.
Screenshots differ only in the clock.
