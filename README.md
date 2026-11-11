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
