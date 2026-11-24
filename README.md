<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq idle checks on FreeBSD, 2026-11-24 (second round)

NEW `16c365b2c189c8e0b0f21e9d8902383a714bb61c` against OLD (v0.1.7)
`d534417e464a5a95823a183a0483a17a230bf3d3`, both cross built on ubuntu-latest
(`#x86_64-freebsd`, `#aarch64-freebsd`) and run in `vmactions/freebsd-vm@main`
(FreeBSD 15.1-RELEASE-p6, 4 CPUs, 6 GiB) in headless sway with `wtype` and
`grim`.  **aarch64 is an emulated machine**: its percentages are of one
emulated core and say little about hardware.

Runs: **45479022804** (x86_64, final), **45484077683** (aarch64, final; the
matrices in the workflow now name only aarch64).  Earlier runs, not used:
45461706313 (x86_64 round one green; the idle job hung somewhere and was
cancelled, the VM returns files only at the end), 45474087770 (idle ran, but
every phase showed 100 percent and the check 4 and 5 frame loops did not run:
an awk `print` rounded the end time), 45478964214 (cancelled for a
5 s sound).

Files: `fb/check.sh` (round one, unchanged), `fb/idle.sh` (checks 2 to 7, in
stages `idle`, `c4`, `c5`, `c7`, each under `timeout`), the workflow; the
evidence is in `results/<arch>/` (`results.txt`, frame logs, screenshots;
`round-one/` holds the round-one screenshots and standard error).

## Verdicts

| # | Check | x86_64 | aarch64 (emulated) |
| --- | --- | --- | --- |
| 1 | Round one on NEW | pass, all eight as in round one | pass, all eight (timing differs only: `/go` cue had finished playing in the screenshot; the tall port-taken shot shows 0 warnings because the holder expired, the plain shot shows the warning) |
| 2 | Idle, 500 files | see below | see below |
| 3 | Idle, light patch | see below | see below |
| 4 | File watch | pass: BROKEN and `3 Cue 3: file not found` 1.12 s after the delete, gone 1.15 s after the restore, `0:05.0` for cue 5 1.12 s after the replace | pass: 2.10 s, 2.72 s, 0.66 s |
| 5 | Light news | pass: `Light output: no network interface` 0.36 s after the failure, gone 0.32 s after the end | pass: 0.64 s, 0.15 s |
| 6 | Clean exit | pass: gone within 0.11 s (first poll) for the 500-cue show and for the lights demo, OLD and NEW | pass: 0.23 to 0.29 s |
| 7 | Byte order mark | pass: standard error empty on both starts, F9 fired GO (Active 1, `0:01.4 left`), file kept its mark; audio device choice not attempted (the VM has no sound card) | same: standard error empty, file kept its mark, F9 fired GO (standby at end of list; the 5 s cue had ended by the screenshot, whose waits are 4 times longer) |

## Idle: percent of one core over 60 s (500 cues, 500 files of 2 s; lights demo)

20 s after opening, `ps -o time=` before and after.  x86_64 also ran a second
round, within 0.03 points of the first.

| | OLD big | NEW big | OLD lights | NEW lights |
| --- | --- | --- | --- | --- |
| x86_64, no ALSA device | 0.63 | 0.05 | 0.28 | 0.07 |
| aarch64, no ALSA device | 9.41 | 0.73 | 3.55 | 0.60 |
| x86_64, null ALSA PCM | 101.2 | 100.7 | 100.2 | 100.0 |
| aarch64, null ALSA PCM | 115.5 | 108.3 | 102.6 | 100.6 |

Round one's `.asoundrc` (`pcm.!default { type null }`, needed for a silent
start without a sound card) makes cinq spin at one full core in both builds,
which hides the fix: the null PCM never blocks (consistent with an audio
thread, not shown by thread listing, `ps -H` printed no rows).  Without any
ALSA device, NEW is near zero and 4 to 13 times below OLD.  The difference
between OLD and NEW survives in the null rows (0.5 and 0.2 points).

## Check 5 method

A VNET jail (`jail -c vnet path=/ persist`, `jexec`) runs cinq with the lights
demo; its `lo0` holds 127.0.0.1 at first.  GO fires cue 1: no warning.  Then
`ifconfig lo0 inet 127.0.0.1 -alias` leaves no IPv4 interface, and 12 s of
frames (about 20 a second natively) with no input show the sidebar turning
to `Light output: no network interface` after 0.36 s; adding the address back
clears it after 0.32 s.  Only that wording was produced; `not sent: ...`
needs an interface that fails to send.
