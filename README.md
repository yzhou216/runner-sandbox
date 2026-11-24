<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# darwin: cinq c9408b4b490e on macos-latest

Branch `cinq/2026-11-24-darwin-checks` of yzhou216/runner-sandbox.  Runner: macos-latest (Darwin 26, aarch64), cinq built natively, `nix build github:yzhou216/cinq/c9408b4b490ef3e26cd5e27048b09981f3f52f48#cinq`, run as cinq.app via `open -n`, keys by System Events, 1024x768 window, standard error to a file.

Runs: 45380429125 (first try: state leaked between my launches, since cinq brings back unsaved changes of a known show; the hint label was not typed), 45383794068 (final: a fresh show and config directory per check, label `sd` typed).  Both green; pictures in the artifact `darwin-macos-latest`, under `checks/<n>-<name>/`.

| # | Check | macos-latest |
| - | - | - |
| 1 | Silent start | pass: cinq.log 0 bytes; status bar `Space: GO / Escape: Panic / ?: help / gt: next tab / l: next list / f: hint` (1-silent/start.png) |
| 2 | Retired keymap | pass: stderr `cinq no longer ships the cua keymap, so vi stands in for it` at config.toml:1:10; sidebar lists `config.toml:1: cinq no...` (Warnings 2); `j`, `o` act: cue 3 added |
| 3 | Hint mode | pass: `f` labels buttons, tabs, fields (sa, sd, sf, ss, se...); typing `sd` added cue 3 (3-hint/added.png); Backspace on empty left the mode (left.png).  Labels cover text here: `sa` hides the colon of `New cue:`, `af` the last letter of `Post-wait`, `sf` touches the `e` of `Add cue` |
| 4 | OSC | pass: `/go` ran cue 1 (Active 1 Hold, 0:53 left); `/cue/99/start` logged `OSC from 127.0.0.1: no cue is numbered "99"`; port held: sidebar `osc_listen: cannot...` (red) |
| 5 | File watching | pass: after mv, State `BROKEN`, header `file not found`, sidebar `1 Tone: file not...`; restored: warning gone (within 4 s each) |
| 6 | Paste | pass: `gg`, `yy`, `p`: `Pasted cue 3`, Action `0:20.0` after 1 s and 5 s |
| 7 | Errors in words | pass: sidebar `Cannot open adir: is a...`; status bar `Cannot save: permission denied` |
| 8 | Closed port | pass: State `SEND FAILED`, sidebar `1 Closed: message not...` |

## Differences and notes

- Standard error, not the sidebar, keeps `(os error N)`: `cannot open ...: Is a directory (os error 21)`, `cannot save ... (os error 13)`, `osc_listen: cannot listen on 127.0.0.1:53000: Address already in use (os error 48)`, `network cue to tcp://127.0.0.1:9: Connection refused (os error 61)`.  The sidebar and status bar texts that fit in the window have none; the sidebar line is cut by the window height, so the full words were not visible, only their start.
- No Local Network prompt appeared at all (loopback only); `ui click Allow` found none.
- Not run: the cross build `#aarch64-darwin`, other Darwin runners.
- No cinq failures found.
