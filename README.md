<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq's keys of Vim on every runner

Tests the commits 68dafc7 to 852f6bd of cinq (the snapshot in `cinq/`): the
vi keymap's Control+o and Control+i for going back and forward, the half and
whole pages, copy, cut, paste, and undo with counts, Control+l for Open, the
cua keymap's Alt with the arrows, and the back and forward buttons of the
mouse.  The snapshot carries a patch for this branch alone that prints each
key that reaches cinq, each key that the keymap parses, the messages that the
keys cause, the rows that a half page measures, and the list, the row, and the
cue selected after every update, to standard error.  `keys/verdict.sh` merges
those lines with the time of each step by `keys/*.sh` and `keys/windows.ps1`
and judges the checks below, so that the results come from cinq's own log and
the screenshots only confirm them.  The show, `keys/show.cinq.toml`, has two
cue lists of five memo cues.  The vi keys run with `keymap = "vi"`, the cua
ones with `keymap = "cua"`, each in a `--config-dir` of its own.  No Space and
no Escape ever reaches cinq (checked in the log of each run).

## Result

Run 43700775347 (commit 445fb61): every check passed on every runner.

| Check | ubuntu-latest | windows-latest | windows-11-arm | macos-latest |
| --- | --- | --- | --- | --- |
| 1 Control+i is Control+i, not Tab, and goes forward after Control+o (`l`, `G`, Control+o, Control+i: row 5, 1, 5) | passed | passed | passed | passed |
| 2 cua: Alt+ArrowLeft goes back, Alt+ArrowRight forward (row 5, 1, 5) | passed | passed | passed | passed, also through System Events |
| 3 Control+d and Control+u move by half the visible rows | passed, 4 rows, 2 moved | passed, 5 rows, 2 moved | passed, 5 rows, 2 moved | passed, 4 rows, 2 moved |
| 4 `y y`, `p p p`, `d d`, `u`, `3 u` (5, 8, 7, 8, 5 rows) | passed | passed | passed | passed |
| 5 Control+l sends Open and shows the file dialog, Control+o does not, cancel closes it | passed | passed | passed | passed |
| 6 back and forward buttons of the mouse | passed, virtual pointer | passed, XBUTTON1 and XBUTTON2 | passed, XBUTTON1 and XBUTTON2 | passed, other buttons 3 and 4 |

Evidence of each result is `verdict.txt` in the artifact `keys-<runner>`,
with `vi.timeline` and `cua.timeline` holding the merged log and the
screenshots next to them: `keys-linux`, `keys-x86_64-windows`,
`keys-aarch64-windows`, and `keys-darwin`.

- Check 1: the raw key is `Character("i")` with `Modifiers(CTRL)`, never
  `Named(Tab)`, and the keymap reads it as Forward.
- Check 3: `page HalfDown ... fit=N moves=N/2`, then the row moves by that
  many.  A window that shows fewer rows than the five cues (900 by 640 for
  the window) is what makes half a page less than the whole list.
- Check 5: the file dialog is the portal's on GNU/Linux
  (xdg-desktop-portal-gtk, started on a session bus of its own, closed by
  killing its window), the common item dialog on Windows (UI Automation
  presses its Cancel button), and the open panel on Darwin (`ui` clicks the
  Cancel that Vision reads).  The window lists and screenshots show it before
  and after.
- Check 6: on GNU/Linux sway headless has no pointer device, but
  wlroots' virtual pointer protocol makes one.  `keys/vptr.c` holds the
  pointer for a few seconds and presses BTN_BACK, BTN_FORWARD, BTN_SIDE, and
  BTN_EXTRA: winit takes BTN_SIDE and BTN_BACK as Back, and BTN_EXTRA and
  BTN_FORWARD as Forward, and cinq did the same.

## Runs

| Run | Commit | Result |
| --- | --- | --- |
| 43697717821 | ca87b70 | windows-latest all passed; windows-11-arm failed (the log fell behind the keys, see below); macos-latest could not cancel the dialog, which then took the keys of the mouse step; ubuntu-latest failed at the out-link `cinq`, which was a directory |
| 43699120602 | da3c70f | both Windows runners all passed; macos-latest failed Control+l's cancel; ubuntu-latest failed on dbus |
| 43700775347 | 445fb61 | all passed |

## Traps

- `wtype g g` types a space between its two arguments, which is GO in cinq;
  type `gg` as one string.
- On windows-11-arm cinq handled each key 20 s or more after it came
  (run 43697717821), so the log lines fell into later steps.  The script now
  waits for cinq to log each key before the next step.
- On Darwin the file dialog belongs to a service of its own, so System Events
  finds no Cancel button in cinq's windows; `ui click --contains Cancel`
  clicks where Vision reads it.  The window of cinq was placed at 60, 60 with
  900 by 640 so that the middle is a fixed point for the mouse buttons.
- Nix's dbus looks for `/etc/dbus-1/session.conf`, which Ubuntu lacks for it:
  `dbus-run-session --config-file=<dbus share>/dbus-1/session.conf`, and the
  portals' share directories in `XDG_DATA_DIRS` before the bus starts.
- `wlrctl pointer click` makes its virtual pointer for less time than cinq
  needs to bind it, and cinq got no button; `keys/vptr.c` holds it.
