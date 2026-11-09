<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq on Darwin: Option+j and Option+k, and the open dialog's filter

Branch `cinq/2026-11-09-darwin-checks`, copy of cinq's branch darwin-checks
at b1f5321 under `cinq/`, with `logging.patch` adding logs of key events and
of the open dialog to that copy alone.  Runner: `macos-latest` (26.7.2,
aarch64), built with `cargo build` and `CINQ_SHARE_DIR=` empty.

Run 42608612584 (run 42605571328 failed at once on a `grep` with no match
and was cancelled):

- Option+j and Option+k do not fire in the vi keymap.  System Events'
  Option+j reaches cinq as `key=Character("j")` with
  `modified_key=Character("∆")`, `text="∆"`, and `modifiers=ALT`, so
  `Press::from_key`, which reads `modified_key`, makes `Char('∆')` with Alt,
  and the keymap finds no action; Option+k likewise gives `˚`.  The control,
  Option+ArrowDown, gave `MoveDown`, and the saved show's order changed only
  for it.
- The open dialog's filter `cinq.toml` did not stop any file from being
  chosen: `other.toml`, `note.txt`, and `show.cinq.toml` all came back from
  the dialog, so `show.cinq.toml` is selectable, and the filter does
  nothing.
- The bare NSOpenPanel steps of `panel.swift` never showed a window, so they
  say nothing.
