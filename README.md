<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq FINAL on GNU/Linux, eight checks

Branch `cinq/2026-11-24-linux-final`, a rerun of `cinq/2026-11-24-linux-checks` with only the cinq commit changed to
`ab9fada06db75110f94d58cd471a965a55448af9` (`verify-drop-cua-regroup`).  Run **45520333765**, green first time on
`ubuntu-latest` (x86_64) and `ubuntu-24.04-arm` (aarch64), no rerun.  The script never fails on a bad check, so standard
error files and screenshots were read for every check on x86_64 and a sample on aarch64.

All eight checks pass on both arches, as in the earlier branch.  Nothing differs.

| # | Check | x86_64 | aarch64 | Evidence |
| --- | --- | --- | --- | --- |
| 1 | Silent start | pass | pass | `1-demo.err`, `1-empty.err` 0 bytes |
| 2 | `keymap = "cua"` | pass | pass | stderr at `config.toml:1:10`, sidebar warning, `o` added cue 4 |
| 3 | Hint mode | pass | pass | labels shown, status bar `Type a label`, Backspace after `a` stays in the mode |
| 4 | OSC | pass | pass | `/go` fired cue 1; cue 99 message; held port: warning `address in use; retrying`, stderr `(os error 98)` |
| 5 | File watching | pass | pass | `BROKEN`, `file not found`; restored, warnings 0 |
| 6 | Paste | pass | pass | `Pasted cue 4`, copy shows `0:08.0` |
| 7 | Errors in words | pass | pass | `Cannot save: permission denied` in status bar and sidebar; stderr keeps `(os error N)` |
| 8 | Closed port | pass | pass | `SEND FAILED`, `Connection refused (os error 111)` |
