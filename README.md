<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq 16c365b2c189 (NEW) against d534417e464a (OLD, v0.1.7) on Windows, idle fixes

Seven checks on `windows-latest` (x86_64) and `windows-11-arm` (aarch64),
4 cores each.  `nix build github:yzhou216/cinq/<sha>#x86_64-windows` and
`#aarch64-windows` for both SHAs are built on `ubuntu-latest` and handed to
the Windows jobs as artifacts.  `ICED_BACKEND=tiny-skia`, 1024x768 desktop,
cinq's window moved to 0,0 at 1024x720 (four times, since the first window of
a job draws larger than the desktop until moved again).  Scripts are in
`win/`: `test.ps1` (round one, unchanged since the earlier branch
`cinq/2026-11-24-windows-checks`), `idle.ps1` (checks 2 and 3), `live.ps1`
(checks 4 to 7), `lib.ps1`, `mkshow.py` (500 cues, 500 distinct WAV files).
Windows jobs run in parallel as separate runners, so the idle measurement is
not disturbed.

## Runs

| Run | What |
| --- | --- |
| 45461637896 | first run: round one green; idle numbers valid; `live.ps1` had a window that did not fit (table regions off) and a quoting bug in the byte order mark step |
| 45464809868 | `live.ps1` fixed: checks 4 to 7 read from here, second idle sample |
| 45468002326 | pointer probe of the audio device list, third idle sample, round one again |

## Verdicts

| Check | windows-latest | windows-11-arm |
| --- | --- | --- |
| 1 Round one, eight checks | pass, same as round one (stderr texts and screenshots identical; run 45461637896, again green in 45464809868 and 45468002326) | pass, same |
| 2 Idle, 500 files, % of one core, mean of 6 | OLD 1.56, NEW 0.43 | OLD 1.67, NEW 0.66 |
| 3 Idle, lights demo, mean of 6 | OLD 0.56 (0.34 to 1.25), NEW 0.12 | OLD 0.43, NEW 0.18 |
| 4 File watch, 500 cues | pass: delete to `BROKEN`, `file not found`, and `2 warnings` in about 1.5 s; put back, gone in 1.4 s; moved over with a 5 s file, Action reads `0:05.0` in 0.9 s; a file written through a handle that shares nothing reads `BROKEN` in under 0.5 s and recovers (`0:05.0`) 0.4 s after the handle is closed | pass: 1.9 s, 1.8 s, 1.4 s; held file `BROKEN` between 1.0 and 1.9 s; recovered 1.4 s after release |
| 5 Light news | not possible, see below | not possible |
| 6 Clean exit | pass: `WM_CLOSE` to the main window ends the process in 29 to 442 ms, 13 closes of NEW and OLD with both shows over runs 2 and 3, none lingering | pass: 30 to 363 ms, 13 closes |
| 7 Byte order mark | pass: Windows PowerShell 5.1 `Set-Content -Encoding utf8` writes `EF BB BF 5B`; NEW starts silent (stderr 0 characters) and F9 is GO (standby 1 to 2, `0:03.0 left`); `pwsh` 7.6 writes none, same result; the audio device list could not be used (below) | pass, same |

Idle detail (`shots2/idle-all.csv`): processor time over 60 s after 20 s of
loading, from `(Get-Process).TotalProcessorTime`, two rounds per run, OLD then
NEW then OLD then NEW for each show.  The timer ticks at 15.6 ms, so a cell
is quantized by about 0.03 percent.  NEW with 500 files is not zero because
the new thread stamps 500 files every two seconds; OLD wakes the whole window
for the same job.

## Differences and notes

- **Check 5.**  Light output went out through loopback, which cannot fail, so
  no failure could be caused without cutting the runner off (disabling every
  adapter but loopback was not tried, as the brief says).  Tried instead, with
  F9 on a light cue after each, then eight seconds of frames: an outbound
  firewall block rule for `cinq.exe` (the packets are dropped silently, `send`
  succeeds) and disabling the Hyper-V adapter (`vEthernet (nat)` on x86_64,
  `vEthernet (Default Switch)` on aarch64).  The status bar stayed at `1
  warning` (the audio device) and no light warning came up; standard error was
  empty.  So a light problem could not be seen end to end on Windows.
- **Check 7, audio device.**  No output device on either runner, so the list
  holds only `System default`; choosing it wrote nothing.  After the session
  the file still reads `EF BB BF`, `[keys]`, `go = ["Space", "F9"]`, and cinq
  starts silent again.  OLD also starts silent with the mark here (stderr 0
  characters) and F9 is GO.
- **Check 6 outlier.**  In the first run, `CloseMainWindow` on the aarch64
  500-cue process took 7.7 s, after the file operations of check 4 and with
  the first window of the job still oversize; the same step took 350 and 363
  ms in the two later runs, and no other close was over 0.45 s.  Not
  reproduced; flagged, not a verdict.
- **Check 4, file open elsewhere.**  Windows refuses to delete or rename over
  a file that a handle holds without `FileShare.Delete`, so the held-open case
  was written in place through an exclusive handle instead: cinq reports
  `BROKEN` while it is held and recovers once it is released, without a key.
- Frames are about 0.45 s apart, so times are good to that.  The screenshots
  of each change are in `shots2/`; the hash of the table and the status bar
  is only a tool for finding the frame.
