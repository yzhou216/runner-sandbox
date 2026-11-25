<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq c9408b4b (verify-drop-cua) on FreeBSD

Eight checks of the 2026-11-24 brief on x86_64 and aarch64 (emulated) FreeBSD
15.1-RELEASE-p6, `vmactions/freebsd-vm@main` on `ubuntu-latest`, with the
cross builds `#x86_64-freebsd` and `#aarch64-freebsd` copied in as Nix store
closures, in headless sway (1280x800; `grim` on a 1280x2400 output for the
whole sidebar), keys by `wtype`, OSC by Python's socket module.
`fb/check.sh` runs the phases; `.github/workflows/freebsd.yml` is the workflow.
The show is two audio cues (generated WAV files) or one network cue; the
demo show was not used (no ffmpeg in the VM).  Both architectures gave the
same results.

Run **45385097492** (commit 6ce7981; the artifacts `checks-*-freebsd` hold
every screenshot and `*.err`).  Earlier: 45380460742 (first pass, ALSA noise on
standard error because the VM has no sound card, so check 1 failed there until
`.asoundrc` named a null device) and 45384938416 (cancelled by a push of
6ce7981 to this branch that I did not make).

| Check | x86_64 | aarch64 | Evidence |
| --- | --- | --- | --- |
| 1 Silent start | pass | pass | `c1.err` empty with a null ALSA device; status bar `Space: GO / Escape: Panic / ?: help / gt: next tab / l: next list / f: hint` |
| 2 Retired keymap | pass | pass | `c2.err` line `config.toml: cinq no longer ships the cua keymap, so vi stands in for it` at `config.toml:1:10`; sidebar lists it as `config.toml:1: ...`; `o` added cue 3 |
| 3 Hint mode | pass | pass | labels two letters; `Add cue` is `sd`, typing it added a cue (`Added cue 3`); Backspace with nothing typed left the mode |
| 4 OSC | pass | pass | `/go` fired cue 1 (Active 1, Beta standby); `/cue/99/start` gave `OSC from 127.0.0.1: no cue is numbered "99"`; port taken gave `osc_listen: cannot listen, address in use; retrying` (no error number in the sidebar) |
| 5 File watching | pass | pass | within 5 s: `BROKEN`, `1 Alpha: file not found`; restored, warning gone |
| 6 Paste | pass | pass | `yy`, `p`: cue 3 shows `0:04.0` |
| 7 Errors in words | pass | pass | `Cannot open c7dir: is a directory`; `Cannot save: permission denied` on the status bar (directory made immutable with `chflags schg`) |
| 8 Network cue | pass | pass | `SEND FAILED`, `1 Closed port: message not sent: connection refused` |

Notes:

- Checks 3 and 6 worked: `wtype` reaches cinq in sway on FreeBSD.
- Hint labels cover text on these fonts (DejaVu Sans Mono): the label of
  `Add cue` hides the end of `New cue:`, others cover the last letters of
  `Basics`, `Levels`, `Light Patch`, and `Post-wait`.  Nothing is unreadable
  except those letters.
- Standard error still shows `(os error N)` in logs (`cinq: cannot open ...: Is
  a directory (os error 21)`, `... (os error 48)`, `... (os error 61)`);
  the sidebar and status bar are in words, as the brief asks.
- The sidebar is cut off at 800 px height (the warning sits beside others
  below); a tall output showed it whole.
- Without a sound card every audio start fails (`OUTPUT FAILED`); a null ALSA
  device made GO work.

## Rerun on cinq 17c2b075 (2026-11-24)

Same workflow, only the cinq commit changed to `17c2b0755ee66d31a99cf4513d8f34c48f09d2f2` (`verify-drop-cua-2`: a thread watches the file stamps and wakes the window on a change; the light thread reports differently).  Run **45397444813**, green first time, no rerun.  All eight checks pass on x86_64 and aarch64.  Check 5: a deleted sound file gave `BROKEN` and `file not found` within the sampling delay (about 4 s), and the warning went when the file was back.  Check 1: standard error 0 bytes.  Check 4 port taken: sidebar `osc_listen: cannot listen, address in use; retrying` on both (aarch64 in c4b-1-taken.png; its later tall shot shows 0 warnings, consistent with the holder having quit and cinq's retry binding).
