<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: GPL-3.0-or-later
-->

# Changelog

Major changes and improvements in each release of cinq, newest first.

## Unreleased

- While a drop-down list is open, the status bar hints the keys that pick and
  close it, such as `Control+y: pick | Control+e: close` in the vi keymap and
  `Enter: pick | F4: close` in the cua keymap, and gives way when the bar is
  short.  In the open list, PageDown and PageUp, or the keys of `page_down` and
  `page_up`, move the highlight by the options in view, Home and End or the
  keys of `first` and `last` move it to the first and last option, as in
  Vim's popup menu.
- `z z`, `z t`, and `z b` scroll the cue list to put the selected row in the
  middle, at the top, or at the bottom of the view in the vi keymap, as in
  Vim, through the new `row_to_middle`, `row_to_top`, and `row_to_bottom`
  actions.  The selection stays on its cue, and the view stops at the ends of
  the list.
- `H`, `M`, and `L` select the first, middle, and last cue row in view in the
  vi keymap, as in Vim, through the new `top_row`, `middle_row`, and
  `bottom_row` actions.  A count before `H` counts rows from the top and
  before `L` from the bottom.  They are jumps for `Control+o` and, unlike the
  motions `j`, `k`, `g g`, and `G`, take no operator.
- `Control+e` and `Control+y` scroll the cue list a row down and up in the vi
  keymap, as in Vim, or as many rows as a count, through the new
  `scroll_down` and `scroll_up` actions.  The selection stays on its cue
  unless it would leave the view, when it moves to the nearest row in view.
  In an open drop-down list the keys keep picking and closing: `pick` and
  `close_menu` act only in a menu, so a key may be bound to one of them and
  to an action of the cue list without a clash.
- The help page leaves out the actions that the keymap in use binds to no
  key, such as `delete` in the vi keymap, so that its table is shorter.
- The help page and the table of cinq(5) list the actions in groups, GO,
  Panic, and standby first, then the moves, then the edits, then the files
  and the window, in place of the order in which actions were added.
- A key that completes or continues no sequence of keys cancels what was
  typed so far and does nothing itself, as Vim beeps and drops the keys, so
  that a mistyped `d x` no longer cuts a cue, and `g j` and `g k` move the
  selection down and up in the vi keymap, as they do in Vim.  The keys of GO
  and Panic still act at once.
- The vi keymap enters the name of the selected cue as Vim enters text:
  `i`, `a`, and `A` put the caret at its end, `I` puts it at its start, and
  `c c` and `S` select all of it, so that typing replaces it.  The new
  `edit_start` and `change_name` actions have no key in the cua keymap, and
  `c` is no operator, so only `c c` begins a change.
- In an open drop-down list, `Control+y` picks the highlighted option and
  `Control+e` closes the list without picking one in the vi keymap, as in
  Vim's popup menu, and `F4` closes it in the cua keymap.  Closing it
  with these keys, unlike with Escape, does not also panic, through the new
  `pick` and `close_menu` actions.
- A count before the keys of `add_below` and `add_above`, such as `3 o`,
  adds that many cues in a row, at most a thousand, as one change for undo,
  and selects the last of them, as the cursor of Vim goes to the last line
  that `3o` inserts.
- `Y` copies the selected cue in the vi keymap, as `y y` does, and takes a
  count.
- In the vi keymap, `d` and `y` take a motion, as in Vim: `d j` cuts the
  selected cue and the next, `y k` copies it and the previous, and `d G`,
  `d g g`, and `d 3 G` reach the last cue, the first, and row 3.  Counts
  before the operator and the motion multiply, so that `2 d 3 j` cuts seven
  cues, and the box above the status bar lists the motions after the
  operator.  The operator key is the doubled key of the binding, so that
  `cut = ["D D"]` makes `D D` and `D j` cut.  A cut of several cues is one
  change for undo.
- Undoing the addition of a cue or a paste below a cue selects the cue that
  was selected when it was made, as the cursor of Vim goes back to where a
  change was made, in place of the cue that follows.
- `/` in the vi keymap and `Control+f` in the cua keymap open a search line
  at the left of the status bar, which selects the first cue after the
  selection whose name or number holds the text typed, in any case, as
  Vim's `incsearch` does, and `?` in the vi keymap searches up the list.
  The search wraps around and ends on the selection.  `Enter` accepts it,
  and on an empty line repeats the last search, and `Escape`, `Backspace` on
  an empty line, or a click closes it, as do going back or forward, moving
  to another field, and opening a show.  `n` and `N`, or `F3` and
  `Shift+F3`, find the next match in the direction of the last search and in
  the other, wrapping around, and a count goes round the matches at most
  once.  Searches are jumps, so that `back` returns to where a search began
  or to the cue before a step to the next match.  `?` no longer shows the
  help page in the vi keymap, as it searches, so that `F1` is its key there.
- Cues can be copied, cut, and pasted with a clipboard of cinq's own that
  outlasts the show it was filled from: `y y`, `p`, and `P` in the vi
  keymap copy and paste, `d d`, `x`, and `Delete` cut, and `Control+c`,
  `Control+x`, `Control+v`, and `Control+V` do the same in the cua keymap.
  A count copies or cuts that many cues and pastes that many times, a paste
  is one change that undo takes back, and a fade pasted together with its
  target targets the new copy, as in QLab.  The status bar tells what was
  cut or copied and names the key that pastes.  The `copy` action works in
  show mode, as Vim yanks in a read-only buffer.  The cua `Delete` key still
  deletes without copying, through the new `copy`, `cut`, `paste_below`, and
  `paste_above` actions, while `delete` has no key in the vi keymap.
- The keys of Vim that move by a page work in the vi keymap: `Control+d` and
  `Control+u` scroll the cue list half a page and move the selection the
  same number of rows, so that it keeps its row on the screen, and
  `Control+f`, `Control+b`, `PageDown`, and `PageUp` scroll a whole page less
  two rows and select the first row in view going down, or the last going
  up.  A count before `Control+d` or `Control+u` sets how many rows they
  move, until the window changes size or another show opens, and a count
  before the whole pages counts them.  The same keys scroll the help page.
  The cua keymap binds `PageDown` and `PageUp` to the whole pages.
- A count typed before the keys of undo and redo turns that many changes, as
  `3 u` does in Vim, and stops at the oldest or newest.  A count before
  `next_tab` shows the inspector tab of that number, as `2 g t` does, and
  before `previous_tab` goes back that many tabs.
- The vi keymap takes more keys of Vim: `x` and `Delete` join `d d` in
  cutting the selected cue, `F1` shows the help page, and `Control+Home`
  and `Control+End` select the first and last cue as `g g` and `G` do, each
  with a count as before.
- `back` and `forward` go back and forward between the places that jumps
  left, as the jump list of Vim does: switching cue lists, `first`, `last`,
  and selecting a cue in another cue list are jumps.  The keys are
  `Control+o` and `Control+i` in the vi keymap and `Alt+ArrowLeft` and
  `Alt+ArrowRight` in the cua keymap, and the status bar names them while
  there is a place to go to.  A jump after going back keeps the places
  ahead, as in Vim, and going back or forward in a text field moves the
  keyboard to the cue list.  The back and forward buttons of the mouse do
  the same from the main window, unless the help page is open.
- In the vi keymap, the key of *open* is now `Control+l`, since `Control+o`
  is Vim's way back through the places jumped to, and Vim's `Control+l`
  only redraws the screen, which cinq never needs.  The cua keymap keeps
  `Control+o`.

## v0.1.4

- While the problems of `config.toml` take a screen of their own on a
  terminal, what cinq and its libraries print meanwhile, such as the
  messages of the sound system, waits and comes out on the main screen once
  the problems clear, or before the problems left when cinq ends, in place
  of being lost when the terminal switches back.  On Windows, only cinq's
  own messages wait.
- `nix build .#demo` builds the demo shows with the sounds, picture, and
  film that they play, which `doc/demo/build.sh` generates with ffmpeg, in
  place of the `demo` branch and its files in Git LFS.
- A tutorial in eight chapters runs the storm show of the demo.
  `nix build .#doc` builds it into `share/doc/cinq/tutorial.pdf`, with
  screenshots that it takes of cinq in a headless sway session, and puts
  the demo beside it in `share/doc/cinq/demo`.  Systems other than
  GNU/Linux build it on a GNU/Linux builder, such as `nix.linux-builder` in
  nix-darwin.  The deb, rpm, Arch, and Darwin packages carry both.
- On GNU/Linux and on Apple silicon, `nix build .#darwin-pkg` builds an
  installer package for Darwin on Apple silicon, which puts `cinq.app` in
  `/Applications` and the `cinq` command, its manual pages, its
  completions, and the tutorial with its demo in `/usr/local`.  The
  package is unsigned.
- The clock of an audio cue follows how far its sound has played, as that
  of a video cue does, so that its fades, its end, and the cues that follow
  it stay with what is heard.  A sound that drifts from the clock of the
  show, or a device that waits before it plays, no longer leaves the cue to
  end before its sound and cut off its last moments.
- A drop-down list too narrow for its options cuts the chosen one short
  with an ellipsis, and its menu grows as wide as the longest option.
- The sound of audio and video cues can play on another audio device than
  the default device of the system.  The Audio device list in the sidebar
  chooses one, and `config.toml` keeps the choice in `audio_device`, by an
  ID that `cinq --audio-devices` lists along with the name of each device.
  A choice applies at once, and the sounds that play go on from where they
  are on the new device once it starts.  The list asks for the devices each
  time it opens, so that a device plugged in since shows.

## v0.1.3

- On Windows, the unsaved changes of shows with the same path on different
  drives, or on different network shares, no longer share one kept file and
  take each other's place.  The drive or share now leads the kept file's
  path, so changes that an earlier version kept for a show on Windows do
  not come back.  Other systems keep their kept files as they were.
- cinq plays up to 1024 sounds at once, where it took the 129th for a
  broken file and kept checking that file again.
- Stopping a cue also cancels the cues of its chain that it would have
  started, even when a cue between them had been fired again or could not
  play.
- GO on a cue that is already running restarts it and cancels the cues that
  its earlier run had yet to start, as QLab does, in place of leaving them
  to play on.
- The help page fills the main window in place of a box over it, and closing
  it gives the window back as it was, with the cue list scrolled where it
  was and the keyboard in the same field.  A field of the main window no
  longer takes the keys typed while it shows.  Besides the keys of the page,
  only those of GO and Panic act.  The text stays in a column at most 960
  pixels wide.  The key of Help opens the page from the main window only,
  since a fullscreen stage window may cover the main window, and does
  nothing while the stage window has the keyboard.  The key of Help also
  opens the page while the menu of a drop-down list is open, and the menu is
  open on the same option when the page closes.
- Opening a show right after switching away from it, while cinq is still
  keeping its unsaved changes, brings back the newest of them and no longer
  older ones.
- A failure to keep unsaved changes now shows on a line of its own among the
  warnings, and no longer replaces the problem of opening or saving a show.
  It also takes back the status note that changes were kept.
- On the stage window, the pictures of earlier cues show around a picture
  of another shape than the window, as in QLab.  The space around it was
  black until the picture began to fade, when they appeared there at once.
- cinq makes ready the sounds that start within ten seconds, instead of those
  of every cue that one GO fires, so that a long chain of auto-continue cues no
  longer opens all its sound files at once.  `ready_ahead` in `config.toml`
  sets that number of seconds, 0 or more and 10 unless the file says
  otherwise, so that owners of small systems can open fewer files at once.
  A change applies as soon as the file is saved.  Sounds that start together
  are always made ready together.
- The clock of a video cue follows how far its sound has played, in place of
  the clock of the show, so that its picture and sound no longer drift apart
  over a long video, as they did by the difference between the clock of the
  audio device and that of the show.  A video without sound, or one whose
  sound has ended, still follows the clock of the show.
- A damaged video file now fails to load with a reason, where it could
  freeze or end cinq, and the files loaded after it load as they would.
- Damaged tags, chapters, or cover art in a video file no longer stop it
  from playing.  cinq reads no tags or cover art, and skips a damaged track
  other than the picture's, which it reports once on standard error.
- QuickTime `.mov` files with AAC sound, which cinq used to refuse, now
  load.

## v0.1.2

- On Windows, a terminal that starts cinq shows what cinq prints, such as
  the problems of `config.toml` and the output of `--help`, and waits until
  cinq closes.  Explorer still starts cinq without a console window.
- Video cues decode with rust_h264, a decoder written in Rust, in place of
  Cisco's openh264, so that building cinq compiles no C++, its program no
  longer needs the C++ library, and it builds on riscv64.  A sample that
  fails to decode now costs no picture but its own.
- On a terminal, the problems of `config.toml` take a screen of their own
  while any are left, as a pager's text does, so that fixing the file brings
  back the prompt and what came before it instead of a cleared screen.
  Problems still left when cinq ends stay on the terminal.
- The Nix packages for other systems go by the systems that they run on, as
  `.#aarch64-darwin`, `.#x86_64-windows`, and `.#aarch64-windows`, in place of
  `.#aarch64-darwin-cross`, `.#x86_64-windows-cross`, and
  `.#aarch64-windows-cross`.
- A picture fading out no longer jumps back to whole when a panic, or
  another fade that stops its cue, takes over; it fades on from where it
  was.
- Video 1280 pixels wide that is cropped below 720 lines, such as
  letterboxed high definition, shows in the colors of high definition.
- On Darwin, cinq follows the changes to `config.toml` in a directory
  reached through a symbolic link, such as one under `/tmp`.
- On FreeBSD and the other BSDs, cinq follows the changes that an editor
  writes into `config.toml` in place, and not only those that rename another
  file over it.
- A cue that GO fires right after a sound or video that could not play
  ended the last running cue no longer starts ahead by the time spent idle.
- Choosing again the file that a running audio or video cue plays, or
  typing its name with a space around it, leaves the cue playing.
- cinq formats `config.toml` each time it is saved, as taplo formats TOML by
  default, but never reorders keys or the items of arrays and inline tables.
  A file that is not TOML stays as it is.
- On GNU/Linux, Nix cross-builds cinq for FreeBSD, as `.#x86_64-freebsd` and
  `.#aarch64-freebsd`, and for GNU/Linux on riscv64, as `.#riscv64-linux`.

## v0.1.1

- Picture cues show a PNG or JPEG on the stage window until a fade, a stop
  cue, or Panic ends them.
- The stage window opens as a window, which the operator can move to the
  display of the audience, and each GO that fires a picture or video cue
  makes it fullscreen on that display.  The fullscreen key turns it back
  into a window until the next such GO.
- Problems in `config.toml` show on standard error each time that cinq
  reads the file, at startup and on each change, with the lines around each
  problem and its place marked, and the sidebar gives the line of each.  On
  a terminal, only the problems of the latest reading stay on screen, so
  that fixing the file clears it, and a file without problems prints
  nothing.
