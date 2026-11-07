<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: the indicatif prototype on GNU/Linux, Windows, and Darwin

Records a prototype of cinq (the local branch `indicatif-prototype`, commit
3c94151, on yzhou216/cinq 01c6d31), which keeps the problems of `config.toml`
at the bottom of the terminal in an indicatif bar, while the log, which
`RUST_LOG` turns on, writes above it through indicatif-log-bridge.  Each
script starts cinq from a terminal with `RUST_LOG=cinq=info`, edits the file
step by step, closes cinq with a problem left, starts it again, and narrows
the terminal.

- `demo-linux.sh`: headless sway on `ubuntu-latest`, with vim editing the
  file in one foot terminal and cinq running in another.  cinq's window
  floats in a gap kept free on the right, so that it does not narrow the
  terminals as it opens.
- `demo.ps1`: `windows-latest` (build 26100), with `cinq.exe` typed into cmd
  in Windows Terminal, the file rewritten by the script, and gdigrab
  recording.
- `demo-darwin.sh`: `macos-latest`, with cinq typed into Terminal through
  System Events after `open -a Terminal`, the file rewritten by the script,
  and `screencapture -V` recording.

Each job uploads its recording and `captions.tsv`, the time of each step
since the recording started, and the captions were burned in afterwards.

## Outcome

Run 42340922193 passed on all three platforms.  While the terminal kept its
width:

- The prompt and every log line stayed, and the report of the latest reading
  stayed below them: it went from two problems to one, left when the file was
  fixed, and came back with the next mistake.
- After cinq closed with a problem left, the report stayed on screen, with the
  prompt below it.

Narrowing the terminal broke the display on all three.  indicatif pads each
line of a bar with spaces to the width of the terminal instead of ending it
with a newline, so foot, Windows Terminal, and Terminal.app rewrapped the
report as one long line at the new width.  The next redraw moved up the rows
that the report took at the old width and left pieces of it behind.  A
report left on screen after cinq closed rewraps the same way at any later
resize.  Locally, tmux did the same, and so did a first run of the GNU/Linux
script in which sway tiled cinq's window beside the terminal, narrowing it
just after the first report was drawn.
