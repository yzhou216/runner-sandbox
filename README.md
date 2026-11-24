<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq c9408b4b490e (v0.1.7 plus "drop the cua keymap") on Windows

Eight checks of cinq `c9408b4b490ef3e26cd5e27048b09981f3f52f48` (branch
`verify-drop-cua`) on `windows-latest` (x86_64) and `windows-11-arm`
(aarch64).  `nix build github:yzhou216/cinq/<sha>#x86_64-windows` and
`#aarch64-windows` are built on `ubuntu-latest` and handed to the two Windows
jobs as artifacts.  `ICED_BACKEND=tiny-skia`, 1024x768 desktop, the window
moved to 0,0 at 1024x720.  `win/test.ps1` runs one phase per check with its own
`--config-dir` and its own copy of a small show (three cues, two generated
WAV files, `win/shows/`); keys go in with SendKeys, GO is bound to F9 as well,
OSC goes from `win/osc.py`.  Verdicts come from screenshots (`shots/`) and
the standard error of cinq (`shots/*-stderr-summary.txt`).  Neither runner has
an audio device, so audio cues read `OUTPUT FAILED` and the sidebar lists
`Default device (System default): not found, retrying`; that is the runner.

On windows-11-arm the "Windows Subsystem for Linux must be updated" window
comes back every minute or so and took the keys of two phases in run
45381935420; `win/closer.ps1` now closes it every 250 ms in the background.

## Runs

| Run | What |
| --- | --- |
| 45380251877 | first run, scripts only; sidebar warnings out of view |
| 45381935420 | sidebar scrolled; arm hint and watch phases covered by the WSL window |
| 45383995207 | closer added; all checks read on both runners (final) |
| 45386038286 | same, with the first phase's window fitted again to show check 1's status bar |

## Verdicts, runs 45383995207 and 45386038286

| Check | windows-latest | windows-11-arm |
| --- | --- | --- |
| 1 Silent start | pass: stderr empty (0 characters); bar `Space: GO \| Escape: Panic \| ?: help \| gt: next tab \| f: hint` | pass, same |
| 2 Retired keymap | pass: stderr `config.toml: cinq no longer ships the cua keymap, so vi stands in for it` at `config.toml:1:10` (line 1); sidebar lists `config.toml:1: cinq no longer ships the cua keymap, so vi stands in for it`; `j` moved to row 2, `o` added cue 4 | pass, same |
| 3 Hint mode | pass: `f` labels buttons, tabs, fields; `sd` (the label of Add cue) added cue 4 ("Added cue 4"); Backspace with nothing typed left the mode | pass, same labels |
| 4 OSC | pass: `/go` moved standby 1 to 2; `/cue/99/start` gave `OSC from 127.0.0.1: no cue is numbered "99"`; port held by Python gave `osc_listen: cannot listen, address in use; retrying`, no number | pass, same |
| 5 File watching | pass: file moved away, within 4 s `BROKEN` and `1 Tone: file not found` in the sidebar on arm, `GO fires nothing: cue 1 (file not found)` on both; put back, warning gone, Warnings (1) | pass |
| 6 Paste | pass: `y y` `p` made cue 4 `Tone` with `0:04.0` in Action, status `Pasted cue 4` | pass |
| 7 Errors in words | pass: directory as show gives sidebar `Cannot open adirectory: permission denied`; save into a directory with a deny ACE gives bar `Cannot save: permission denied` | pass |
| 8 Closed port | pass: `SEND FAILED`, sidebar `1 Closed port: message not sent: connection timed out` | pass, same |

## Differences and notes

- Standard error keeps the error number where the interface does not: `cannot
  open ...: Access is denied. (os error 5)`, `cannot save ...  (os error 5) at
  path ...`, `osc_listen: cannot listen on 127.0.0.1:53000: Only one usage of
  each socket address ... (os error 10048); trying again every 2 seconds`.
  The sidebar and the status bar are in words, as the brief asks; if the brief
  meant standard error too, this fails.
- Windows says `permission denied` for opening a directory and `connection
  timed out` for a closed port on the same computer (the one second limit ends
  before Windows reports a refusal), not `is a directory` or `connection
  refused`.
- Hint labels cover text on Windows: each label sits over the end of the text
  to its left (`New cue:` reads `New cu`, `Add cue` reads `Add cu`, `Basics`,
  `Levels`, `Light Patch`, `Time & Loops`, `Post-wait`).  Labels differ
  between hint sessions as cues are added, so a label is read from the screen
  first.
- No Windows Security (firewall) prompt came up on windows-11-arm for the
  listener of cinq on 127.0.0.1 (checked at start and 3 s later, no window titled
  `Windows Security`); python was allowed by a rule before it held the port.
- The first phase of each run drew cinq's window at its own default size,
  larger than the desktop, until the window was moved again.
- `$env:PROCESSOR_ARCHITECTURE` reads AMD64 on windows-11-arm inside the
  emulated shell; the artifact of the aarch64 build was the one run there.

## Rerun on cinq 17c2b075 (2026-11-24)

Same workflow, only the cinq commit changed to `17c2b0755ee66d31a99cf4513d8f34c48f09d2f2` (`verify-drop-cua-2`: a thread watches the file stamps and wakes the window on a change; the light thread reports differently).  Run **45397407912**, green first time, no rerun.  All eight checks pass on windows-latest and windows-11-arm.  Check 5: a deleted sound file gave `BROKEN` and `file not found` within the sampling delay (about 4 s), and the warning went when the file was back.  Check 1: standard error 0 bytes.  Same differences as before (os error 5 on stderr, timed out for the closed port, hint labels covering text).
