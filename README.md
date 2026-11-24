<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq audio checks, 2026-11-24

Branch `cinq/2026-11-24-audio-checks` of yzhou216/runner-sandbox; cinq 861183a5cd82 (`nix build github:yzhou216/cinq/861183a5cd82b7e6a460f498e7e2102b7172446f#cinq`) on ubuntu-latest.

Runs: 45395016716 failed (my script ran the program's directory, not its binary); 45397676219 passed but its check 8 edited cue 9 (selection had moved), so discarded; **45401496386 is the result** (pass; its first analysis took the wrong beep, re-analysed from the artifact with the fixed `analyze.sh`).

Method: private PulseAudio, null sinks `house` (default) and `booth` (own server, so killing it removes the device); ALSA PCMs `alsa:house`, `alsa:booth`; monitors recorded; a 440 Hz mono tone at -3 dBFS (so RMS = cue level - 6 dB - send); GO is F9, Panic F10. Times are from the key's wtype call to the measured sound, aligned by a calibration beep, so accurate to about 0.2 s.

| # | Check | Measured | Verdict |
|---|---|---|---|
| 1 | cue at -6 dB, default device | house -12.0 dB RMS (expected -12.0), booth silent | pass |
| 2 | sends House 0, Booth -10 | house -12.0, booth -22.0 | pass |
| 3 | fade to -20 over 3 s | house -18.0 to -26.0, booth -28.0 to -36.0, linear from GO, flat from +3.0 s | pass |
| 4 | loop 2-4 s, then devamp | intro -23, loop -9 until devamp (+7.3 s), left at the pass end (+8.0 s), ending -17 for 2 s, silent from +10.0 s | pass |
| 5 | Stop; Panic F10 | Stop: silent within 0.2 s; Panic: fade to silence 1.0 s (a one-second fade by design, not a fraction of a second) | pass, Panic takes 1.0 s |
| 6 | level typed -6 to -12 on Levels | house -12.0 to -18.0, booth -22.0 to -28.0 (change -6.0 dB) | pass |
| 7 | booth device gone | cue 9 (Booth only) `OUTPUT FAILED`, sidebar warns `alsa:booth: stopped`; cue 7 on both keeps house at -6.0 dB | pass |
| 8 | clear last send | status `Changed cue 8 to play nowhere`; house silent 0.1 s after its send cleared, booth 0.2 s after the last | pass |

Notes: a cue that still plays on another output does not read `OUTPUT FAILED`; the sidebar and the Levels tab warn instead. While I worked, something else edited and committed `audio/record.sh` on the branch (a separate booth server, the cue 9 fix); I kept it.

## cinq 861183a5 audio checks, 2026-11-24

See `audio/` for the scripts and the workflow `audio.yml` (run 45404626989, ubuntu-latest, success).

Local: GNU/Linux, private PulseAudio and headless sway (out6); runner numbers are from the artifact of 45404626989.
Levels are RMS of the monitor of each sink; a -3 dBFS tone at cue level L reads L-6.

| # | Check | Local | ubuntu-latest |
|---|-------|-------|---------------|
| 1 | -6 dB cue, default device | -12.0 (steady) | -12.0 |
| 2 | sends House 0, Booth -10 | -12.0 / -22.0 | -12.0 / -22.0 |
| 3 | fade to -20 over 3 s | -18 to -26.0, linear, done at +3.0 s | same |
| 4 | loop 2-4 s, devamp | loop -9.0 until devamp (+7.3 s), ending -17.0 for 2 s, silent at +10 s | same |
| 5 | Stop / Panic | Stop silent in <0.1 s; Panic 0.9 s ramp (-6 dB per 0.1 s) | 0.1 s; 1.0 s |
| 6 | -12 typed while playing | -12.0 to -18.0 and -22.0 to -28.0 (moved by the change) | same |
| 7 | device gone | pass, see below | pass |
| 8 | last send cleared | status bar `Changed cue 8 to play nowhere`, silent in <0.2 s | same |

Check 7: PulseAudio moves the streams of an unloaded sink to the default sink, so cinq saw no fault.  Booth therefore lives on a PulseAudio server of its own, which the script kills.  A cue sending to both outputs keeps playing on House (-6.0), the sidebar warns `alsa:booth: cannot open, retrying` within 7 s (not yet at 3 s), and the Levels tab says `Booth (cannot open)`.  The cue itself does not read `OUTPUT FAILED`: by the manual that needs every send to fail.  A cue 9 sending only to Booth reads `OUTPUT FAILED` and `silent: output failed` in the Active list.

Notes: Panic fades out over about a second rather than cutting, a design point to confirm against "a fraction of a second".  GO moves the selection to the next cue, which the first script missed.  The beep of calibration falls in the first second of the House recording on the runner (the analysis first skipped it).
Files: audio/record.sh, make-show.sh, analyze.sh, report.sh; artifacts hold screenshots and the recordings.
