<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq's master at 35afa6b: the keys, the new checks, and the audio

Tests cinq on ubuntu-latest, windows-latest, windows-11-arm, and macos-latest
(the audio checks on the three that had the means in the earlier audio branch,
`cinq/2026-11-16-audio-keys`).  The snapshot in `cinq/` is 35afa6b ("tutorial:
name the keymap that it uses and correct four statements", committed on
2026-11-16 at 05:36:36 -0800), unpacked from `git archive` and committed at that
date, so that the flake is a Git one.  Two edits are for this branch alone:

- A logging patch.  It carries over the lines of `cinq/2026-11-15-vim-keys`
  (`SANDBOX <time> ...`: each raw key, each parsed `Press` with its action, each
  message, the state after each update, the measures of a page, the status bar
  hints, the keys of a drop-down list) and the lines of the audio branch (`K`
  for keys no widget took, `U` for the state after each update, `O` around each
  opening of a device).  The code moved since 16723cb and 6bddf30, so four hunks
  of the keys patch were written again (`dispatch.rs`, `paging.rs`,
  `paging/kept.rs`, `status_bar.rs`).  The state now holds `shown`, `held`,
  `file_held`, `tab`, `output`, `unapplied`, and the outputs with their
  channels.  New lines: `focus ... slots=[(in the inspector, has the keyboard)]
  -> target` and `focused widget order=N id=... x= y=` for every Tab (in
  `focus_order.rs`), and `helptext first_steps=... rows=Action = keys ; ...`
  whenever the help page is built.
- `doCheck = false` in `cinq/flake/cinq.nix`: tests that measure the system's
  fonts failed in the Nix builds of the two earlier branches on ubuntu-latest.

The scripts are `keys/` (the earlier keys branch, changed and extended below)
and `audio/` (the earlier audio branch, run from here by the same workflow,
`.github/workflows/master.yml`).  The verdicts come from cinq's own log, and
the screenshots only confirm them.  No Space ever reaches cinq (GO is F5 in the
audio phases), and Escape reaches it only in the two phases of audio check 2,
`pulse/two` and `darwin/two`, where nothing else plays (checked in every log).

## Result

Run 43906701488 (commit 6f9a57c, snapshot 35afa6b): all 11 jobs green.  Every
check passed except one row of check 16, which is a failure of cinq (below).
Two bugs of the verdict scripts made the run's own `verdict.txt` wrong in three
places, and the scripts were fixed after the run (commit after 6f9a57c): BSD
sed has no `\|`, so three lines of check 14 failed on Darwin (`verdict.sh`
now uses `sed -E`), and the Windows and Darwin `default` phases of the audio
branch stopped cinq while it wrote a log line, so the last `U` line was cut and
the Levels check read a send table without Right (`verdict.py` now skips a line
that does not end with `})`).  The fixed scripts judge the same artifacts, and
the tables are their verdicts; the sound of those two phases was heard at
-16.0 and -28.0 dBFS in the run itself, which the run's own verdict passed.

### Keys (artifacts `keys-linux`, `keys-x86_64-windows`, `keys-aarch64-windows`, `keys-darwin`)

| Check | ubuntu-latest | windows-latest | windows-11-arm | macos-latest |
| --- | --- | --- | --- | --- |
| 1 Control+i is not Tab, goes forward after Control+o | passed | passed | passed | passed |
| 2 cua Alt+ArrowLeft and Alt+ArrowRight | passed | passed | passed | passed, also System Events |
| 3 Control+d and Control+u move half the visible rows | passed | passed | passed | passed |
| 3 three Control+f in a row move three pages | passed | passed | passed | passed |
| 4 `y y`, `p`, `d d`, `u`, `3 u` | passed | passed | passed | passed |
| 5 Control+l opens the file dialog, Control+o does not | passed | passed | passed | passed |
| 6 mouse back and forward buttons | passed (4 buttons) | passed (XBUTTON1, 2) | passed | passed (buttons 3, 4) |
| 7 `d 2 j` cuts 3 cues ("Cut 3 cues"), `p` pastes them | passed | passed | passed | passed |
| 8 `/` and `?` with `n`, `N`, Control+o back | passed | passed | passed | passed |
| 8 wrap notices at the bottom and the top | passed | passed | passed | passed |
| 9 `z z`, `z t`, `H`, `L`, Control+y, `55 G` | passed | passed | passed | passed |
| 9 Control+e scrolls one row, selection stays in view | passed | passed | passed | passed |
| 10 drop-down list: Control+y picks, Control+e closes, hints | passed | passed | passed | passed |
| 11 `d u` does not undo, Control+c after `2 d` cuts nothing | passed | passed | passed | passed |
| 12 search line: ArrowUp, caret at the end, Control+u, Control+c | passed | passed | passed | passed |
| 13 first Tab reaches the inspector's first field, then Name | passed | passed | passed | passed |
| 14 Enter in an output's Name applies "Front" and leaves, `j` moves | passed | passed | passed | passed |
| 14 Enter in its Channels applies "1-2" and leaves, `k` moves | passed | passed | passed | passed |
| 15 help page: `dd`, `gg`, `cc` joined, first steps paragraph | passed | passed | passed | passed |
| 16 Enter in an empty File field sends ChooseFile, chooser opens | passed | passed | passed | passed |
| 16 the chooser closes with the platform's cancel | passed | passed | passed | passed |
| 16 status bar hints Enter once the File field has the keyboard | failed | failed | failed | failed |
| no Space and no Escape reached cinq in any phase | passed | passed | passed | passed |

Changes of the scripts for master (the checks and their results above):

- Check 9: cinq now counts a row as in view when more than half of it shows
  (`rows_in_view`), so `verdict.sh` computes the top and bottom rows that way.
  Control+e, which skipped a row on 16723cb, passed on all four.
- Check 10: Tab from the cue list reaches the inspector first, so the
  toolbar's list of cue types is the ninth Tab (8 fields of a memo cue), where
  it was the first.  The keyboard stays on the list after Control+y, so the
  second Enter needs no Tab.
- Check 13: the log line `focus ... slots=[...] -> target=2 in_inspector=Some(true)`
  and `focused widget order=2` (x 269, y 412 on the 900 by 640 window): the first
  of the 8 fields of the inspector (places 2 to 9), none had the keyboard before.
  The second Tab is `id=Some(Id(Custom("name")))` at place 3.  The ninth is place 0,
  the toolbar's list.
- Check 14: `l`, `c c` (Name field, `Focus(Name(..))`), `Front` typed (each letter a
  `SettingTyped`, the name shows `Front` at once), Enter (`Focus(Cues)`, the
  field left), `j` (action Next, output 2, no `SettingTyped`, name still
  `Front`); then Tab three times (Name, Device list, Channels at place 3), `1-2`
  typed (outputs unchanged while typing), Enter (`Right@1-2`, `Focus(Cues)`),
  `k` (output 1, no `SettingTyped`).
- Check 15: the `helptext` line holds `First = gg, Control+Home`, `Cut = dd, x,
  Delete`, `Copy = yy, Y`, `ChangeName = cc, S`, no `d d`, and the first
  steps paragraph ("First steps: Space is GO, ... cc names it, and u undoes the
  last change.").  `search-h-help.png` on each runner shows the paragraph under
  the title.
- Check 16: a show of two memo and audio cues, the audio one with no file.
  `j`, Tab three times (place 4, `id=Some(Id(Custom("file")))`), Enter:
  `message ChooseFile(CueId(2))`, and a window "Choose sound file" (the portal
  on GNU/Linux, UI Automation on Windows, Vision OCR on Darwin) that Cancel closes.

### Failure of cinq: the hint of Enter in the File field comes one key late

After the third Tab the File field has the keyboard, but the status bar still
reads "Tab: next field | Control+c: leave | F1: help".  The hint "Enter: choose
a file | Control+c: leave" appears only after the next update, which is the
Enter itself.  The same log lines on all four runners (`file.timeline`):

```
message Focus(Next)
focus backward=false slots=[(false, false), (false, false), (true, false), (true, true), (true, false), ...] -> target=Some(4) in_inspector=Some(true)
focused widget order=4 id=Some(Id(Custom("file"))) x=269.02344 y=495.40002 width=156.47656 height=30.8
message Held(true)
message FileHeld(false)
status hints="Tab: next field | Control+c: leave | F1: help" room=89 menu=false
  ... (Enter)
message ChooseFile(CueId(2))
message Held(true)
message FileHeld(true)
status hints="Enter: choose a file | Control+c: leave" room=89 menu=false
```

`Held` and `FileHeld` ask the widgets in a task that runs next to the one that
moves the keyboard, so they read the keyboard's place before the move: each
answer is one Tab behind (`Held(false)` after the first Tab, `Held(true)` after
the second, and `FileHeld(false)` after the third, with the field focused).  It
is cinq's, not the script's; `file-16-focused.png` shows the old hint with the
File field's blue border.  Enter itself works.

### Audio (artifacts `audio-jack`, `audio-pulse`, `audio-alsa`, `audio-windows`, `audio-darwin`)

| Check | ubuntu-latest | windows-latest | windows-11-arm | macos-latest |
| --- | --- | --- | --- | --- |
| 1 JACK ports past nine | passed (out_0..11 to playback_1..12, 16 ports recorded) | skipped, no JACK | skipped, no JACK | skipped, no JACK |
| 2 two devices, Panic stops both | passed (-13.5, -19.5 dBFS) | skipped, one VB-CABLE | skipped, no VB-CABLE or FlexASIO | passed (BlackHole 16ch, 2ch) |
| 3 clamp, 16-bit | passed | passed (FlexASIO Int16) | skipped, no FlexASIO | skipped, Core Audio takes float |
| 3 clamp, 24-bit | passed (S24_LE); packed S24_3LE skipped, cinq says it cannot play it | passed (Int24) | skipped | skipped |
| 3 clamp, 32-bit and float | passed | passed (Int32) | skipped | skipped |
| 4 interface while a device opens | passed (7 keys in 4 opens of 1.99 s, updates 1 ms after) | skipped, no slow device | skipped | skipped |
| 5 one output on 1-2 of the default device plays | passed (PulseAudio's) | passed (VB-CABLE, default and by ID) | skipped | passed (BlackHole 16ch, default and by ID) |
| 5 outputs view keys `l j o d d p`, Control+o | passed | passed | skipped | passed |
| 5 Levels tab: three Tabs, `-12`, Enter set Right to -12.0 | passed | passed | skipped | passed |
| 5 send heard 12 dB down (-16.0 and -28.0 dBFS) | passed | passed | skipped | passed |

On windows-11-arm the earlier audio branch had no job, and nothing here installs
a virtual audio device for it.  The Levels tab check changed with the Tab order: on
master three Tabs reach the send of Right (places 2, 3, and 4 of the order, with
the sends of Left and Right at 3 and 4), where it took five.  Tab now selects the text of the field, so the
select-all before `-12` is no longer needed but does no harm.

## Runs

| Run | Commit | Result |
| --- | --- | --- |
| 43906701488 | 6f9a57c | snapshot 35afa6b; 11 jobs green; results above |

Local runs of `keys/linux.sh`, `audio/pulse.sh`, and `audio/jack.sh` on this
machine (the same Nix tools and the same build of the snapshot) came first and
gave the same verdicts on GNU/Linux, which is how the Tab counts and the new
checks were found before the run.

## Traps

- `wtype g g` types a space between its two arguments, which is GO in cinq; type
  `gg` as one string.
- A log that cinq writes while a script stops it (Stop-Process, kill) can end
  in half a line; ignore a last line that is cut.
- BSD sed (macOS) has no `\|`; use `sed -E`.
- Tab order is not the order of the window: the inspector first, then the rest,
  so counts of Tabs from earlier branches no longer hold.  Read the `focused
  widget` lines.
- The `Held` and `FileHeld` messages lag one key behind the keyboard (see the
  failure above), so a log line of `held=` after a Tab shows the key before.
- On GNU/Linux wtype's keymap gives F5 the physical key code that winit calls
  Escape (`key: Named(F5)`, `physical_key: Code(Escape)`); the logical key is F5.
- Nix's dbus looks for `/etc/dbus-1/session.conf`, which Ubuntu lacks for it:
  `dbus-run-session --config-file=<dbus share>/dbus-1/session.conf`.
- `wlrctl pointer click` makes its virtual pointer for less time than cinq
  needs to bind it; `keys/vptr.c` holds it.
- On windows-11-arm cinq handles each key 20 s or more late, so the script waits
  for cinq to log each key before the next step.
- BlackHole needs `sudo killall coreaudiod` after the install; the sandbox's
  PulseAudio needs `ALSA_PLUGIN_DIR` to find the pulse plugin of ALSA.
