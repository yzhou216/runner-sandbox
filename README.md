<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# darwin: cinq OLD vs NEW on macos-latest

Branch `cinq/2026-11-27-darwin-loc`.  Workflow, scripts, and data copied from `cinq/2026-11-24-darwin-final`; only the cinq commit became a matrix over the build.  Runner: macos-latest (Darwin 26.7.2, aarch64), cinq built natively with Nix.

- OLD: `e304bd4e581af53fa32c5ae9a598c2d655130e61`
- NEW: `efa362c0f5193bc3bd73bbc4494284fa6c6429ce`

Run **46111148692**, green first time for both jobs (OLD 12 min, NEW 14 min), no rerun.  Artifacts `darwin-old` and `darwin-new`, read by hand (screen texts, pictures, cinq.log), since the script's verdict step never fails.

| # | Check | OLD | NEW |
| - | - | - | - |
| 1 | Silent start | pass: stderr 0 bytes, status bar names Space, Escape, help, hint | same |
| 2 | Retired keymap | pass: warning at config.toml:1:10 on stderr; `j`, `o` act | same |
| 3 | Hint mode | pass: label `sd` added cue 3, Backspace left | same |
| 4 | OSC | pass: `/go` ran cue 1 (0:57.8 left); `no cue is numbered "99"`; port held warning | same |
| 5 | File watching | pass: BROKEN and `file not found`, cleared on restore | same |
| 6 | Paste | pass: `Pasted cue 3`, Action `0:20.0` at once and after 5 s | same |
| 7 | Errors in words | pass: `Cannot open adir`, `Cannot save: permission denied` | same |
| 8 | Closed port | pass: `SEND FAILED` | same |

## Differences between OLD and NEW

None.  All ten cinq.log files match byte for byte except temporary file names (`.tmpXXXXXX`).  Screen texts match after dropping the clock and coordinates, apart from 6-paste/at-once, where the accessibility dump of OLD held a few more sidebar labels; the two pictures are identical.

Notes: the script's marks `2-cua:cua`, `6-paste:loading`, and `8-network:not sent` count 0 on both builds (stale search words, not cinq).  No Local Network prompt appeared.
