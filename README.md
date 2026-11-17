<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq's packages in distribution containers

`github:yzhou216/cinq/97af215a58520fe2a30ae5d2bffb98f90800a9e6` (cinq 0.1.4,
branch `wip/light-cues`) built as `#deb`, `#rpm`, and `#archlinux` natively
with Nix on `ubuntu-latest` (x86_64) and `ubuntu-24.04-arm` (aarch64), then
installed with each distribution's own tool in a `docker run --privileged`
container on the runner of its architecture.

## What runs

`.github/workflows/distro.yml`, with the scripts in `distro/`:

- **build** builds the three packages and hands them over as `pkgs-<arch>`.
- **test** (17 jobs) runs `distro/run.sh`: install the package on the bare
  image, `cinq --version`, `ldd`, the man pages, the desktop file; where cinq
  starts, install sway, wtype, grim, and python3 from the distribution, start
  headless sway, run `show.toml` (instruments on universes 1, 2, and 999,
  light cues at 20, 50, 70, 100, 80, 30, and 40 percent with a 4 s fade, a
  burst of four network cues, a destination that refuses, a stand-in console
  that goes down and comes back) beside `listen.py` and `fakeeos.py` (from
  branch `cinq/2026-11-13-eos-offline`), and `analyze.py` summarizes.  Each
  GO is a `wtype` space 3 to 6 s after the last; its time is the call plus
  the 0.3 s that `wtype -s 300` waits.

## Results (run 43693560570; run 43695248629 on 449e1af repeated it, all green)

glibc of Nixpkgs is 2.44, and the packages say `libc6 (>= 2.44)` or
`glibc >= 2.44`.

| Distribution | glibc | x86_64 | aarch64 |
| --- | --- | --- | --- |
| debian:stable (13, trixie) | 2.41 | install refused: `Depends: libc6 (>= 2.44) but 2.41-12+deb13u4 is to be installed`; forced, loader: ``libm.so.6: version `GLIBC_2.43' not found`` | same |
| debian:testing | 2.43 | refused (`libc6 (>= 2.44) but 2.43-7`); forced: ``version `GLIBC_2.44' not found`` | same |
| ubuntu:24.04 | 2.39 | refused; forced: ``version `GLIBC_2.43' not found`` | same |
| ubuntu:25.10 | 2.42 | refused; forced: ``version `GLIBC_2.43' not found`` | same |
| ubuntu:rolling (26.10) | 2.44 | installs, runs, all checks pass | same |
| fedora:latest (45) | 2.44 | installs, runs, all checks pass | same |
| fedora:rawhide (46) | 2.44.9000 | installs, runs, all checks pass | same |
| opensuse/tumbleweed | 2.44 | **zypper refuses**: `nothing provides 'libgcc' needed by the to be installed cinq-0.1.4-1`; with `rpm --nodeps` and its libraries by hand (`libasound2` etc.) it runs, all checks pass | same |
| archlinux:latest | 2.44 | installs, runs, all checks pass | no official image, skipped |

On the refused distributions, the other dependencies resolve: only glibc
stands in the way (`ldd` lists `libasound.so.2` too until the library is
installed).

Where cinq ran (9 containers of run 1, 2 of run 2), every result was
identical:

- 12 of 12 DMX checks pass: after cue 1 U1 slot 1 = 51, U2 = 128, U999 = 255;
  after the fade U1 = 179 and 204, U2 = 255, U999 = 77; after cue 3 U1 = 128
  and 204, U999 = 102 and slot 512 = 255, as sACN and as Art-Net, with
  Art-Net Port-Addresses 0, 1, and 998.
- sACN priority 100, source name `cinq`, discovery lists 1, 2, 999 every
  10.0 s (first one at start).
- Senders: 127.0.0.1 (loopback copy) and the container's eth0 address
  (172.17.0.2): same counts.
- Idle: 10 packets per 10 s per universe, sender, and protocol.  Fade: 153
  packets in 4 s (38.2 per second) per universe 1, sender, and protocol.
- Network cues: the burst of four arrived 12.9 to 13.3 s (4 messages 0.1 s
  apart) on one TCP connection (UDP datagram in between); int 7, float 2.5,
  string "hello world", string "12" and int -3 decoded as written.  GOs
  3 s or more apart made a new connection each (4 over the run).
- Refused destination: `cinq: network cue to 127.0.0.1:3999: Connection
  refused (os error 111)` 5 to 13 ms after GO, once; sidebar Warnings (1)
  shows it (`refused.png`).  With the stand-in down: the same text for
  127.0.0.1:3037 within 11 ms; the cue after it came back arrived at once.
- `group joins that failed: 19`: listen.py's join of every group on 0.0.0.0
  after eth0 gives `Address already in use`, harmless.

## Findings

- **rpm on openSUSE**: the dependency `libgcc` is not a capability that
  Tumbleweed provides (it provides `libgcc_s1`), so `zypper install` cannot
  install the rpm on either architecture.
- Container images leave out man pages (Ubuntu, Arch, Tumbleweed); the files
  are in the package lists (`files.txt`), and Debian and Fedora installed
  `cinq.1`, `cinq.5`, and `cinq-show.5` under `/usr/share/man`.  The desktop
  file and `cinq.xml` landed everywhere.  man pages were not rendered where
  `man-db` could not find them.
- On Fedora and Arch the binary is found as `/usr/sbin/cinq` (a symlink to
  `/usr/bin`).
- Old distributions need a glibc of 2.44, which no release other than
  Ubuntu 26.10, Fedora 45 and 46, Tumbleweed, and Arch has at this date.

## Rerun on 1a21980 (2026-11-16)

cinq `1a2198013d7e4a7122b90c4141ca5285096a5df9` (cinq 0.1.4, `wip/light-cues`,
rebased onto the latest master and cleaned up) in place of 97af215.  Only
`CINQ` in `.github/workflows/distro.yml` changed.  Run **43985739821**
(commit 39df31f): both builds and all 17 test jobs green.  Artifacts read:
the 17 `<distribution>-<arch>` ones; `analyze.py` and `report.txt` come from
them.

| Distribution | Old (43693560570, 43695248629) | New (43985739821), x86_64 and aarch64 alike |
| --- | --- | --- |
| debian:stable (glibc 2.41) | refused: `libc6 (>= 2.44) but 2.41-12+deb13u4`; forced: `GLIBC_2.43` not found | the same, same version strings |
| debian:testing (2.43) | refused (`2.43-7`); forced: `GLIBC_2.44` not found | the same |
| ubuntu:24.04 (2.39) | refused; forced: `GLIBC_2.43` not found | refused (`2.39-0ubuntu8.9`); `GLIBC_2.43` not found |
| ubuntu:25.10 (2.42) | refused; forced: `GLIBC_2.43` not found | refused (`2.42-0ubuntu3.1`); `GLIBC_2.43` not found |
| ubuntu:rolling (2.44) | installs, all checks pass | installs, 12 of 12 DMX checks PASS |
| fedora:latest (45) | installs, all checks pass | installs, 12 of 12 PASS |
| fedora:rawhide (2.44.9000) | installs, all checks pass | installs, 12 of 12 PASS |
| opensuse/tumbleweed | zypper refuses (`nothing provides 'libgcc'`); `rpm --nodeps` plus libraries: all pass | the same refusal (exit 4, `cinq-0.1.4-1.x86_64` and `.aarch64`); then 12 of 12 PASS |
| archlinux:latest (x86_64) | installs, all checks pass | installs, 12 of 12 PASS |

In the 9 containers where cinq ran, the numbers equal the old ones: sACN
priority 100, source `cinq`, discovery lists 1, 2, 999 (first at start, then
every 10.0 s), Art-Net Port-Address 0, 1, 998, senders 127.0.0.1 and
172.17.0.2, idle 10 packets in 10 s per universe, sender, and protocol, a
fade of 153 packets in 4 s (38.2 per second), the burst of four network cues
at about 12.9 to 13.3 s on one TCP connection (UDP datagram in between), GOs 3 s
or more apart one connection each, the refused destination warned once 4 to 10
ms after GO (`Connection refused (os error 111)`), the same text for
127.0.0.1:3037 while the stand-in was down, and the cue after its return
arrived at once.  Three `Io error: Broken pipe (os error 32)` lines when
sway ends.

Difference: none.  cinq still prints version 0.1.4.

## Rerun on 0ed5d6c (2026-11-17)

cinq `0ed5d6c41dbe29b3140f29238dae014a0a8baa81` (cinq 0.1.4, `wip/light-cues`,
rebased onto the rewritten master; every commit kept its tree) in place of
1a21980.  Only `CINQ` in `.github/workflows/distro.yml` changed.  Run
**44054350718** (commit 4332bf2): both builds and all 17 test jobs green
(09:24 to 09:54 UTC).  Artifacts read: the 17 `<distribution>-<arch>` ones;
the `summary.txt` of each `out.tar` comes from `analyze.py`.

| Distribution | Old (43985739821), x86_64 and aarch64 alike | New (44054350718), x86_64 and aarch64 alike |
| --- | --- | --- |
| debian:stable (glibc 2.41) | refused (`2.41-12+deb13u4`); forced: `GLIBC_2.43` not found | the same, same version string; `GLIBC_2.43` and `GLIBC_2.44` not found |
| debian:testing (2.43) | refused (`2.43-7`); forced: `GLIBC_2.44` not found | the same |
| ubuntu:24.04 (2.39) | refused (`2.39-0ubuntu8.9`); `GLIBC_2.43` not found | the same (also `GLIBC_2.44` in the forced run) |
| ubuntu:25.10 (2.42) | refused (`2.42-0ubuntu3.1`); `GLIBC_2.43` not found | the same (also `GLIBC_2.44`) |
| ubuntu:rolling (2.44) | installs, 12 of 12 PASS | installs, 12 of 12 PASS (`cinq 0.1.4`) |
| fedora:latest (45) | installs, 12 of 12 PASS | installs, 12 of 12 PASS |
| fedora:rawhide | installs, 12 of 12 PASS | installs, 12 of 12 PASS |
| opensuse/tumbleweed | zypper refuses (`nothing provides 'libgcc'`); `--nodeps` plus libraries: 12 of 12 PASS | the same refusal; then 12 of 12 PASS |
| archlinux:latest (x86_64) | installs, 12 of 12 PASS | installs, 12 of 12 PASS |

In the 9 containers where cinq ran: sACN priority 100, source `cinq`,
discovery lists 1, 2, 999 (first at start, then every 10 s: -6.3, 3.7, 14.6,
24.6 ... on fedora:latest), Art-Net Port-Address 0, 1, 998, senders 127.0.0.1
and 172.17.0.2, idle 10 packets in 10 s per universe, sender, and protocol, a
fade of 153 packets in 4 s (38.2 per second), the burst of four network cues
at 12.93 to 12.97 s on one TCP connection (UDP datagram in between), GOs 3 s
or more apart one connection each, the refused destination warned 4 to 10 ms
after GO (`Connection refused (os error 111)`; 5 ms to 127.0.0.1:3037 while
the stand-in was down), and the cue after its return arrived 3 to 4 ms after
GO.  Three `Io error: Broken pipe (os error 32)` lines when sway ends.

Difference: none.  cinq still prints version 0.1.4.  `ldd` of the forced install prints both `GLIBC_2.44` and `GLIBC_2.43` on debian:stable and the two Ubuntu images (only `GLIBC_2.44` on debian:testing); the first rerun quoted one line per image.
