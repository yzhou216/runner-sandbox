<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq's keys of Vim on every runner

Tests the Vim key series of cinq on ubuntu-latest, windows-latest,
windows-11-arm, and macos-latest.  The snapshot in `cinq/` is 16723cb, the
rewritten tip of e1d5853 (the same tree, committed on 2026-11-15 at 10:33:04
-0800, except for two test files: e1d5853's status bar test
`the_key_of_cancel_stays_in_a_bar_of_1024_pixels...` measures the system's
monospace font and failed in the Nix build on ubuntu-latest, run 43756303288,
where 16723cb counts 103 columns instead).  The snapshot carries a patch for
this branch alone that prints each key that reaches cinq, each key that the
keymap parses, the messages that the keys cause, the rows that a page measures,
the scrolls that a selection causes, the status bar hints, the keys of a
drop-down list, and the list, the row, the cue, the search line, and the notice
selected or shown after every update, to standard error.  `keys/verdict.sh`
merges those lines with the time of each step by `keys/*.sh` and
`keys/windows.ps1` and judges the checks below, so that the results come from
cinq's own log and the screenshots only confirm them.  The shows are
`keys/show.cinq.toml` (two lists of five memo cues), `long.cinq.toml` (thirty
cues), `sixty.cinq.toml` (sixty cues), and `names.cinq.toml` (ten cues with
words, for the search line).  The vi keys run with `keymap = "vi"`, the cua
ones with `keymap = "cua"`, each phase in a `--config-dir` of its own.  No
Space and no Escape ever reaches cinq (checked in the log of every phase).

## Result

Run 43773983274 (commit 86854ae, snapshot 16723cb): every check passed on every
runner except two rows of check 9, where cinq skips a row (see below).  The
workflow's own `verdict.txt` still held one false failure on each Windows runner and
seven on Darwin, which were bugs of `verdict.sh` (a Cancel action that cinq does
not produce for Control+c with keys waiting, and `sed` alternation that BSD sed
lacks); the fixed script, committed after the run, judges the same artifacts
as in the table.

| Check | ubuntu-latest | windows-latest | windows-11-arm | macos-latest |
| --- | --- | --- | --- | --- |
| 1 Control+i is not Tab, goes forward after Control+o | passed | passed | passed | passed |
| 2 cua Alt+ArrowLeft and Alt+ArrowRight | passed | passed | passed | passed, also System Events |
| 3 Control+d and Control+u move half the visible rows | passed | passed | passed | passed |
| 3 three Control+f in a row move three pages | passed, 3 pages of 2 rows | passed | passed | passed |
| 4 `y y`, `p`, `d d`, `u`, `3 u` | passed | passed | passed | passed |
| 5 Control+l opens the file dialog, Control+o does not | passed | passed | passed | passed |
| 6 mouse back and forward buttons | passed | passed | passed | passed |
| 7 `d 2 j` cuts 3 cues ("Cut 3 cues"), `p` pastes them | passed | passed | passed | passed |
| 8 `/` and `?` with `n`, `N`, Control+o back | passed | passed | passed | passed |
| 8 wrap notices at the bottom and the top | passed | passed | passed | passed |
| 9 `z z`, `z t`, `H`, `L`, Control+y, `55 G` centers | passed | passed | passed | passed |
| 9 Control+e scrolls one row, selection stays in view | failed twice | failed twice | failed twice | failed twice |
| 10 drop-down list: Control+y picks, Control+e closes, hints | passed | passed | passed | passed |
| 11 `d u` does not undo, Control+c after `2 d` cuts nothing | passed | passed | passed | passed |
| 12 search line: ArrowUp, caret at the end, Control+u, Control+c | passed | passed | passed | passed |

Evidence of each result is `verdict.txt` (the run's own) in the artifact
`keys-<runner>`, with the `*.timeline` files holding the merged log and the
screenshots next to them: `keys-linux`, `keys-x86_64-windows`,
`keys-aarch64-windows`, and `keys-darwin`.  Highlights, the same on all four
runners unless a number is given:

- Check 3, Control+f: three `page WholeDown` lines of 2 rows each (the list
  shows 4 or 5 rows), and each landing starts where the one before ended, from
  0 to about 191.6 pixels (rows are 31.8 pixels).
- Check 8, wraps: from row 1, `n` after `?1` goes to row 21 with "Search
  continued at the bottom", and `N` from there to row 1 with "Search continued
  at the top"; `n` and `N` that do not wrap show no notice.
- Check 9, `55 G`: row 55 selected and the list scrolled to the middle (offset
  1655.7 against a middle of 1655.7 on ubuntu-latest, 1663.2 on both Windows
  runners, 1657.7 on macos-latest).
- Check 12, with the names show: `/Bl`, Enter (row 2), `/Br`, Enter (row 4),
  `gg`, `/B` (row 2), ArrowUp recalls "Br" (row 4), `i` makes "Bri" (so the caret
  was at the end), Control+u clears the line (row 1), `Bo` selects row 6,
  Control+c closes the line and returns to row 1.
- Check 11: after `2 d` the pending keys are `2`, `d`, and after Control+c
  they are empty, with no Cut and the rows unchanged (30).
- Check 10: Enter opens the list with the hints "Control+y: pick | Control+e:
  close | F1: help", Control+y picks the next option (Audio to Video), and
  Control+e closes it with Video kept.

### Failure of cinq: Control+e skips a row

On the 60-cue show, from row 1: Control+e once scrolls 31.8 pixels and selects
row 2, as expected.  The list then reports its scroll as 32 (whole pixels), so
the second Control+e scrolls to 63.8 and selects row 4, not row 3, and the
third scrolls to 95.8 and selects row 5, not row 4.  A row that the scroll
clips by 0.2 pixels counts as out of view.  It is cinq's rounding, not the
script's (the log lines are `measured view=... scrolled=32 -> offset=63.80002
row=4`), identical on all four runners.

## Runs

| Run | Commit | Result |
| --- | --- | --- |
| 43700775347 | 445fb61 | six checks of the 852f6bd snapshot, all passed on all four runners |
| 43731026095 | a2979f6 | checks 7 to 12 of the 06bd3a0 snapshot, run by an earlier agent |
| 43756303288 | b355009 | snapshot e1d5853; attempt 1 stayed queued for 47 minutes and was cancelled, attempt 2 failed the Nix build on ubuntu-latest (the status bar test above); Windows and Darwin ran, with the same results as the last run |
| 43773983274 | 86854ae | snapshot 16723cb; all jobs green, results above |

## Traps

- `wtype g g` types a space between its two arguments, which is GO in cinq;
  type `gg` as one string.
- On windows-11-arm cinq handled each key 20 s or more after it came, so the
  script waits for cinq to log each key before the next step, and sends a key
  again that left no line.
- On Darwin the file dialog belongs to a service of its own, so `ui click
  --contains Cancel` clicks where Vision reads it.  The window of cinq is
  placed at 60, 60 with 900 by 640.
- BSD sed (macOS) has no `\|` in basic expressions; `verdict.sh` uses `sed -E`
  there.
- Control+c with keys waiting makes the keymap drop them without an action,
  while with the search line open it gives the Cancel action.
- Nix's dbus looks for `/etc/dbus-1/session.conf`, which Ubuntu lacks for it:
  `dbus-run-session --config-file=<dbus share>/dbus-1/session.conf`.
- `wlrctl pointer click` makes its virtual pointer for less time than cinq
  needs to bind it; `keys/vptr.c` holds it.
- A run of this workflow waited 47 minutes with every job queued, even on
  ubuntu-latest, with no incident on githubstatus.com; cancelling and rerunning
  it did not help, and attempt 2 started only after about two more hours.
