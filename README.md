<!--
SPDX-FileCopyrightText: 2026 runner-sandbox contributors

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# scrollreversed: Darwin runtime tests

Tested [scrollreversed](https://github.com/yzhou216/scrollreversed), cross-built
on GNU/Linux, on Darwin runners.

- `test.yml`, on `macos-15`, `macos-latest`, and `macos-15-intel`: installs the
  binaries and the launchd agent, grants Accessibility through `TCC.db`, and
  checks with `probe.swift` that wheel events come out reversed and gesture
  events untouched under several configurations.
- `nix.yml`, on `macos-15`: `nix build` and `nix flake check` of the source
  snapshot in `src-tree/`.

Outcome: everything passed, in runs 41784727047 and 41785065226.  A running
process ignores an Accessibility grant until it restarts, even after tccd
restarts and `com.apple.accessibility.api` is posted, so the daemon exits for
launchd to restart it.
