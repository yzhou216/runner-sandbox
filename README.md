<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: the problems screen on GNU/Linux, Windows, and Darwin

Records cinq (yzhou216/cinq 01c6d31 with the uncommitted work of the local
branch `problems-screen`) showing the problems of `config.toml` on a screen of
their own, the alternate screen that full-screen programs use.  Each reading
draws its problems there in place of those before; the first reading without
any switches back to what the terminal showed before; problems left when cinq
ends, by its window or by Control+c, come back to that screen once.

Each script shows the file in a terminal, starts cinq there, edits the file
step by step, narrows the terminal while problems show, closes cinq with a
problem left, starts it again, and presses Control+c.

- `demo-linux.sh`: headless sway on `ubuntu-latest`, with vim editing the
  file in one foot terminal and cinq running in another, cinq's window
  floating in a gap kept free on the right.
- `demo.ps1`: `windows-latest` (build 26100), with `cinq.exe` typed into cmd
  in Windows Terminal, the file rewritten by the script, and gdigrab
  recording.
- `demo-darwin.sh`: `macos-latest`, with cinq typed into Terminal through
  System Events after `open -a Terminal`, the file rewritten by the script,
  and `screencapture -V` recording.

Each job uploads its recording and `captions.tsv`, the time of each step
since the recording started, and the captions were burned in afterwards.

## Outcome

Run 42355917900 passed on all three platforms, and the terminals showed the
same at each step:

- The problems replaced the terminal's contents, and each save redrew them.
- Fixing the file brought back the output of `cat` or `type` and the command
  line that started cinq.
- Narrowing the terminal while problems showed redrew them at the new width,
  with miette wrapping its messages to it, and left nothing behind.
- Closing cinq's window, and pressing Control+c, each left the remaining
  problem below the command line, followed by a new prompt.
