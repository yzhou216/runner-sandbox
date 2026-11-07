<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: a console program that Explorer starts without a console

Checked `cinq.exe`, cross-built on GNU/Linux from `cinq/` (a snapshot of
uncommitted work on 90cd7ce), on `windows-latest` with `console.ps1`.  cinq
is now a console program whose manifest sets Windows' console allocation
policy to `detached`, so that Explorer starts it without a console, while a
shell runs it in its own console and waits for it, and its standard error,
with the problems of `config.toml`, shows there.

The script starts `cmd.exe` from Explorer as a control, then `cinq.exe` from
Explorer, counting the `conhost.exe` children of each, one for every console
that Windows makes for a process.  It then types `cinq.exe` into `cmd` in a
console window, edits `config.toml` while cinq runs, and closes cinq, with a
screenshot of each step.

## Outcome

Run 42318185909 (6a2f199) passed on Windows build 26100:

- `cmd.exe` started from Explorer had 1 console, and `cinq.exe` started from
  Explorer, by `explorer.exe`, had none (`1-explorer.png` shows its window
  alone).
- Typed into `cmd`, cinq drew both problems of the file on standard error in
  that console (`2-console.png`), which Windows Terminal hosts on this image,
  with colors and Unicode frames.  Fixing one left the other alone on the
  screen (`3-one-left.png`), fixing both cleared it (`4-fixed.png`), and
  deleting the closing bracket showed that the file is not TOML, with the
  mark past the end of the file (`5-not-toml.png`).
- `cmd` showed its prompt again only once cinq was closed (`6-closed.png`).

The `screenshots` artifact of the run holds the screenshots.
