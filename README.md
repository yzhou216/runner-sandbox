<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq ab9fada0 (verify-drop-cua-regroup) on FreeBSD

Rerun of `cinq/2026-11-24-freebsd-checks` with only the cinq commit changed to
`ab9fada06db75110f94d58cd471a965a55448af9`.  Run **45520348012**, green first
time, no rerun.  Eight checks on x86_64 and aarch64 (emulated) FreeBSD 15.1.
Every verdict equals the earlier README's: all pass on both.

| Check | x86_64 | aarch64 |
| --- | --- | --- |
| 1 Silent start | pass (c1.err 0 bytes) | pass (0 bytes) |
| 2 Retired keymap | pass | pass |
| 3 Hint mode | pass | pass |
| 4 OSC | pass | pass (see below) |
| 5 File watching | pass | pass |
| 6 Paste | pass | pass |
| 7 Errors in words | pass | pass |
| 8 Network cue | pass | pass |

Difference in evidence only: on aarch64 the check 4 screenshot after `/go`
shows Active 0, standby moved to 2 and Alpha's state empty, so the 4 s cue had
already ended under emulation (the earlier run showed Active 1); GO fired.
