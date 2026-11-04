<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: GPL-3.0-or-later
-->

# scrollreversed

Darwin daemon reversing mouse or trackpad scrolling

`scrollreversed` reverses the scroll direction of either the mouse or the
trackpad, so that each can scroll its own way despite the single natural
scrolling setting in System Settings.

## How it works

A Quartz event tap in the login session classifies each scroll event by the
device that sent it, the way Darwin picks between the Mouse and Trackpad
settings, so a Magic Mouse counts as a mouse.  Events from senders outside
the I/O Registry are classified by gesture phase instead.  Reversal negates
every delta the event carries, including the one in the embedded HID event
that WebKit reads.

## Building

On Darwin, `nix build` builds the package.  On GNU/Linux, it cross-compiles
for Apple silicon against Apple's SDK; the linker signs the binary ad hoc,
which is all Darwin needs to run it.  Either way, the package holds the
binary and both manual pages.

In `nix develop`, `cargo build --release --target aarch64-apple-darwin`
cross-compiles from GNU/Linux too, `cargo test` runs the platform-independent
tests, `cargo xtask man` renders the manual pages into `target/man`, and
`nix fmt` formats everything.

## Installing

```sh
sudo install -m 755 result/bin/scrollreversed /usr/local/bin/
cp launchd/io.yiyuzhou.scrollreversed.plist ~/Library/LaunchAgents/
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/io.yiyuzhou.scrollreversed.plist
```

On first start, allow `scrollreversed` under System Settings > Privacy &
Security > Accessibility.  The daemon then exits so that launchd restarts it
with access.  It logs to `/tmp/scrollreversed.log`.

The permission is tied to the binary's signature, so after installing a new
build, remove `scrollreversed` from the Accessibility list, run
`launchctl kickstart -k gui/$(id -u)/io.yiyuzhou.scrollreversed`, and allow
it again.

To uninstall:

```sh
launchctl bootout gui/$(id -u)/io.yiyuzhou.scrollreversed
rm ~/Library/LaunchAgents/io.yiyuzhou.scrollreversed.plist
```

## Usage

See scrollreversed(1) and scrollreversed(5).

## License

GPL-3.0-or-later
