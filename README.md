<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq FINAL ab9fada0 on Windows

Rerun of the eight checks of `cinq/2026-11-24-windows-checks` on
`ab9fada06db75110f94d58cd471a965a55448af9` (`verify-drop-cua-regroup`) on
`windows-latest` and `windows-11-arm`.  Only the cinq SHA in the workflow
changed.  Run **45520325545**, green first time, no rerun.  Screenshots and
standard error read from the `checks-*` artifacts; a selection is in `shots/`.

| Check | windows-latest | windows-11-arm | Earlier |
| --- | --- | --- | --- |
| 1 Silent start | pass, stderr 0 characters | pass | same |
| 2 Retired keymap | pass, stderr and sidebar name `config.toml:1` | pass | same |
| 3 Hint mode | pass, stderr empty, cue 4 added by label, Backspace left | pass | same |
| 4 OSC | pass, no cue 99 message; port taken retries, no number | pass | same |
| 5 File watching | pass, `BROKEN`, `file not found`, back again | pass | same |
| 6 Paste | pass, cue 4 `Tone`, `Pasted cue 4` | pass | same |
| 7 Errors in words | pass, `Cannot save: permission denied`; directory open denied | pass | same |
| 8 Closed port | pass, `SEND FAILED`, `connection timed out` | pass | same |

No verdict differs.  The earlier notes hold unchanged: standard error keeps
the os error numbers, Windows reports `permission denied` and `connection
timed out`, and no audio device exists on either runner.
