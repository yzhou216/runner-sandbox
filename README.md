<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq's Darwin installer package

This session tested `cinq-0.1.3-aarch64-darwin.pkg`, the unsigned flat
package that `nix build .#darwin-pkg` makes on GNU/Linux with cpio, mkbom
(bomutils 0.2 with hogliux/bomutils#34), and xar under faketime.  It holds
two components: `cinq.app` for `/Applications`, owned root:admin, and the
`cinq` link, manual pages, and completions for `/usr/local`, owned
root:wheel, both with `overwrite-permissions="false"`.  The package itself
is cinq's, under GPL-3.0-or-later.

[`darwin-pkg.yml`](.github/workflows/darwin-pkg.yml) installs it with
`installer`, compares the directories that the payloads share with the
system before and after ([`stat.sh`](stat.sh)), reads the receipts, runs
`cinq --version` from `PATH`, finds the manual pages with `man -w` and the
zsh completion through `fpath`, opens the app, installs again over it,
opens the package in Installer, and opens a copy marked as downloaded by
Safari.

## Results

Run 42957668774 passed on `macos-15` (Darwin 15.8.2, 24H232) and `macos-26`
(Darwin 26.7.2, 25G332); `macos-latest` was also 26.7.2 in runs
42956243152 and 42956305333.  `macos-14` no longer starts ("The
configuration 'macos-14-arm64-us-default' is not supported").

- `installer` reports "The install was successful", and a second install
  "The upgrade was successful".  `pkgutil` lists `io.yiyuzhou.cinq` in
  `Applications` and `io.yiyuzhou.cinq.cli` in `usr/local`, with every file.
- Directories that were there keep their owner and mode, including
  `/usr/local/bin` and `/usr/local/share` owned by runner:admin with mode
  775.  Missing ones, such as `/usr/local/share/zsh/site-functions`, are
  made root:wheel with mode 755.
- `cinq --version` runs through `/usr/local/bin/cinq`; `man -w` finds
  cinq(1), cinq(5), and cinq-show(5); zsh autoloads `_cinq`.  The app
  starts with `open`.
- `pkgutil --check-signature` says "no signature", and `spctl --assess
  --type install` rejects the package ("no usable signature").  Installer
  logs the trust failure but installs from the command line regardless.
- A double click on the package opens Installer at "Welcome to the cinq
  Installer".  A copy with `com.apple.quarantine` set by Safari opens
  Gatekeeper's dialog instead: "cinq.pkg" Not Opened, "Apple could not
  verify "cinq.pkg" is free of malware", with Move to Trash and Done.
  `open` waits for an answer there, so the workflow runs it in the
  background.
- `sudo installer -pkg` installs the quarantined copy; the installed app,
  program, and link carry no extended attributes, so the app starts with
  no Gatekeeper prompt.
- `codesign --verify --strict` fails on the bundle ("code has no resources
  but signature indicates they must be present"), since only the linker
  signs the program ad hoc; the app starts anyway.
- After Done in that dialog, Privacy & Security shows ""cinq.pkg" was
  blocked to protect your Mac." with an Open Anyway button in its Security
  section, on both versions (run 42958503999).  System Settings showed no
  buttons to `entire contents` of System Events, so
  [`scroll.swift`](scroll.swift) scrolls the pane with CoreGraphics wheel
  events to bring the section into the screenshot.

## Recording

[`cinq-darwin-pkg-install.mp4`](cinq-darwin-pkg-install.mp4) (2 minutes
33 seconds) shows run 42999176642 on `macos-latest` (Darwin 26.7.2):
[`demo.sh`](demo.sh) installs the package as a user would, while ffmpeg
records the screen, and [`compose.sh`](compose.sh) burns its captions in.
Safari downloads the package from this branch, a double click in Finder
meets Gatekeeper's block, Privacy & Security offers Open Anyway, Installer
installs it, cinq opens from Applications with no prompt, and Terminal
shows `/usr/local/bin/cinq`, `cinq --version`, the manual pages, and both
receipts.

Two steps differ from a user's, as the captions say:

- The password dialog of Open Anyway, which `coreautha` shows, refused
  every password that a script gave it, as keys from CoreGraphics or as
  values set through accessibility: `opendirectoryd` logged
  `ODErrorCredentialsInvalid`, though `dscl . -authonly` accepted the same
  name and password.  Terminal clears the quarantine with `xattr -d`
  instead, as Open Anyway would.
- The workflow allows `system.install.software`, the only right that
  Installer asked `authd` for, so Installer asks for no password.

Lessons for later recordings:

- [`ui.swift`](ui.swift) clicks text that Vision reads from a screenshot,
  since System Events sees no buttons in System Settings.  Vision reads
  buttons side by side as one line ("CancelAllow") and gives a part of a
  line the whole line's box, so the helper places the part by its share of
  characters.  It read "cinq" in Finder as "cing", and "cin" matches
  "Macintosh HD".
- Safari took 25 seconds to start the first time.  `osascript` quitting a
  Safari with a sheet open waits two minutes; `pkill` does not.
- `screencapture -V` ran on far past its limit once the screen stood still,
  even after SIGINT, so the recording comes from ffmpeg's `avfoundation`
  device (Homebrew installs ffmpeg in about a minute), which ends on SIGINT.
  Its output stayed empty for about a minute at first, so the script
  measures the video's lead over its own clock from the file's length.
- `dscl . -passwd` cannot set the runner's password, which has a
  SecureToken, and no guess found it.  `sysadminctl -addUser` made an
  administrator whose password only `dscl . -passwd` set for sure.
- In AppleScript, `dialog` and `label` cannot name loop variables inside
  `tell process`.
