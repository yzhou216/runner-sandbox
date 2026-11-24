<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq c9408b4 on GNU/Linux, eight checks

Branch `cinq/2026-11-24-linux-checks`, workflow `.github/workflows/checks.yml`, script `checks/check.sh`.
cinq is built natively with `nix build github:yzhou216/cinq/c9408b4b490ef3e26cd5e27048b09981f3f52f48#cinq`
(the demo from `#demo`) on `ubuntu-latest` (x86_64) and `ubuntu-24.04-arm` (aarch64), and run in headless sway
(pixman, tiny-skia, a null ALSA device) with wtype and grim; GO is bound to F9 as well.

Runs: 45380063576 and 45383299462 (earlier scripts; my `wtype f f` typed a Space, which is GO, between keys, so the
screenshots of checks 2, 3, 6 were unreliable), **45386122829** (final, green on both arches; screenshots of every
check read on x86_64, and 3, 4, 5 on aarch64 as well).

| # | Check | x86_64 | aarch64 | Evidence (run 45386122829) |
| --- | --- | --- | --- | --- |
| 1 | Silent start | pass | pass | `1-demo.err`, `1-empty.err` 0 bytes, with the storm demo and with no show; status bar `Space: GO / Escape: Panic / ?: help / gt: next tab / l: next list / f: hint` |
| 2 | `keymap = "cua"` | pass | pass | stderr `config.toml: cinq no longer ships the cua keymap, so vi stands in for it` at 1:10 (the value on line 1); sidebar lists it as `config.toml:1: ...`; `j` moved down, `o` added cue 4 (`2-cua-2-o.png`) |
| 3 | Hint mode | pass | pass | labels beside buttons, tabs, fields; `sd` added a cue; Backspace after `a` stayed in the mode, a second left it |
| 4 | OSC | pass | pass | `/go` fired cue 1 (`4-osc-1-go.png`); `/cue/99/start` gives sidebar `OSC from 127.0.0.1: no cue is numbered "99"`; port held: sidebar `osc_listen: cannot listen, address in use; retrying` (no errno), stderr keeps `(os error 98)`; warning gone after the holder quit |
| 5 | File watching | pass | pass | within 4 s: `BROKEN`, `1 Rain: file not found`; restored, warning gone |
| 6 | Paste | pass | pass | `yy`, `p`: copy shows `0:08.0` at once (`6-paste-1-now.png`) |
| 7 | Errors in words | pass | pass | sidebar `Cannot open adir: is a directory`; `Control+s` into a 555 directory: status bar `Cannot save: permission denied`, sidebar `Cannot save show.cinq.toml: permission denied`; stderr keeps `(os error N)` |
| 8 | Closed port | pass | pass | State `SEND FAILED`, sidebar `1 Send: message not sent: connection ...`; stderr `network cue to tcp://127.0.0.1:9: Connection refused (os error 111)` |

Notes:

- Hint labels cover the end of the text to their left on DejaVu at 1100x700: `Add cue` reads `Add cu`, `Post-wait` reads `Post-wai`,
  `New cue:` loses its tail, `Time & Loops` its last letter.  Same on both arches.
- Label letters depend on the state (Add cue was `sf` with cues playing, `sd` when idle).
- The storm demo opens a black stage window over the main window in sway (floating, same size).
- Nothing differs between x86_64 and aarch64.
