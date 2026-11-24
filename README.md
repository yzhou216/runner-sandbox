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
