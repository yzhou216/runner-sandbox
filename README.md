<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: demo of cinq.exe as a console program on Windows

Records a video of cinq at yzhou216/cinq 01c6d31, `cinq: run as a console
program on Windows`, which makes `cinq.exe` a console program whose
manifest sets a detached console allocation policy.  A GNU/Linux job
cross-builds it with `nix build ./cinq#x86_64-windows-cross`, and
`demo.ps1` records the desktop of `windows-latest` (build 26100) with
`ffmpeg -f gdigrab`, writing the time of each step to `captions.tsv`.

## Outcome

Run 42334582098 passed:

- Explorer started `cinq.exe` with no console window and no `conhost.exe`
  child.
- Typed into cmd, `cinq.exe --version` printed there, and `cinq.exe`
  drew the problems of `config.toml`, which redrew on each save, cleared
  once the file was fixed, and returned when a closing bracket went
  missing.  cmd showed its prompt only after cinq closed.

Run 42333935578 failed: pressing Enter in Explorer never started
`cinq.exe` within 90 seconds, and the MP4 that ffmpeg left behind had no
moov atom.  The second run starts the program with `explorer.exe <path>`
and records to MKV, which stays readable when the recorder is cut off.
