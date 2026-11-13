<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: GPL-3.0-or-later
-->

# cinq (CINQ Is Not QLab)

Minimal cue-based show controller, the
[free](https://www.fsf.org/about/what-is-free-software) alternative to QLab

## Features

- **Built not to stall.**  The sounds of the next GO are opened and decoded
  ahead of time, and loading, saving, and keeping unsaved changes all run in
  the background, so a slow disk does not hold up GO or Panic.
- **Your work survives.**  Unsaved changes are kept as you make them, through
  closing the window or a crash, and come back as one change that undo takes
  back.  Undo covers every edit, and nothing ever asks before acting.
- **Keyboard first.**  vi and CUA keymaps, rebindable in a file that reloads
  as you save it.  GO and Panic act from anywhere, even halfway through
  another key sequence, apart from Space while a text field has the keyboard.
- **Audio cues** in WAV, FLAC, Ogg Vorbis, MP3, or AAC, with start and end
  times, seamless loops, and levels that change while they play.
- **Video cues** in H.264 and MP4, with their sound, which cinq decodes
  itself with rust_h264 rather than handing them to another player, and
  **picture cues** in PNG or JPEG, which hold until something stops them.
  Both show on a stage window that each GO of one makes fullscreen.
  A fade that stops a picture or a video, and Panic, fade it out along with
  its sound, to black or to the picture that shows on under it.
- **Light cues** that fade dimmers over Art-Net, each instrument keeping
  its level until a later light cue names it.
- **Fade, stop, wait, and memo cues**, chained by auto-continue and
  auto-follow, with pre-waits and post-waits.
- **Panic** fades everything out in a second, and a second Panic stops at
  once.
- **Show mode** locks editing during a performance.
- **Several cue lists**, and a warnings list that says why any cue cannot
  play.
- **Readable show files** in TOML, which `cinq SHOW` opens, easy to diff and
  keep under version control.

## Building

### Requirements

#### General

- Cargo
- [scdoc](https://git.sr.ht/~sircmpwn/scdoc), for the manual pages that a
  build lays out, as below

#### GNU/Linux and the BSDs

- pkg-config
- ALSA
- Wayland
- libxkbcommon
- D-Bus
- XDG desktop portal with a file chooser

### Installing

`cargo install cinq` builds cinq from crates.io and installs its program
alone, in the `bin` directory of Cargo, which needs no scdoc.

A build lays out what cinq installs besides its program in the directory
that `CINQ_SHARE_DIR` names, which is `target/share` in the repository, as
the share directory of a prefix holds it: the manual pages, completions for
bash, zsh, fish, and Nushell, and, for GNU/Linux and the BSDs, the desktop
entry with its icon and the type of show files.  The Nix package installs it
all.  Elsewhere, after `cargo build --release`, copy `target/release/cinq`
into the `bin` directory of the prefix, which is often `/usr`, and what
`target/share` holds into its `share` directory, then run
`update-mime-database` with its `share/mime` directory.

Builds for Darwin also lay out the app bundle, `cinq.app`, in
`target/Applications`, with all but the program, which goes into its
`Contents/MacOS`.  Each Nix package of cinq goes by the name of the system
that it runs on, which names the native build too, as `.#x86_64-linux` does
on x86_64 GNU/Linux.  On GNU/Linux, `nix build` cross-builds it for the
other systems as well: `.#x86_64-linux`, `.#aarch64-linux`, and
`.#riscv64-linux` for GNU/Linux, `.#x86_64-freebsd` and `.#aarch64-freebsd`
for FreeBSD, `.#aarch64-darwin` for Apple silicon, and `.#x86_64-windows`
and `.#aarch64-windows`, which build `cinq.exe`, for Windows.  There
`.#deb`, `.#rpm`, and `.#archlinux` build packages for other distributions,
which need a glibc as new as that of Nixpkgs.  On GNU/Linux and on Apple
silicon, `.#darwin-pkg` builds an installer package for Darwin on Apple
silicon.  It puts `cinq.app` in `/Applications` and the `cinq` command, the
manual pages, the completions, and the tutorial with its demo in
`/usr/local`.  The package is unsigned, so Darwin opens a copy that a
browser downloaded only after Open Anyway in the Privacy & Security
settings, while `sudo installer -pkg` installs it from a terminal.  The
tutorial takes its screenshots in sway, which runs on GNU/Linux alone, so
other systems build it on a GNU/Linux builder, such as the virtual machine
of `nix.linux-builder` in nix-darwin.

## Configuration

See [cinq(5)](man/cinq.5.scd).

## Show files

See [cinq-show(5)](man/cinq-show.5.scd).

## License

cinq is free software: you can redistribute it and/or modify it under the terms
of the GNU General Public License as published by the Free Software Foundation,
either version 3 of the License, or any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
