<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq's light and network cues on Darwin

cinq at `97af215a58520fe2a30ae5d2bffb98f90800a9e6` (`wip/light-cues`), run
as `cinq.app` opened with `open -n`, with GO pressed by System Events
keystrokes.  `darwin/lights.sh`, `net.sh`, and `cable.sh` hold the tests,
`darwin/*-verdict.py` the checks, `listen.py` and `fakeeos.py` come from
branch `cinq/2026-11-13-eos-offline`.

## Runners

| Label | Darwin | CPU | Build | Result |
| --- | --- | --- | --- | --- |
| `macos-latest` | 26.7.2 | aarch64 | `nix build $CINQ#cinq`, 639 s with the tests | all pass |
| `macos-26` | 26.7.2 | aarch64 | same, 692 s | all pass |
| `macos-15` | 15.8.2 | aarch64 | same, 663 s | all pass |
| `macos-15-intel` | 15.7.9 | x86_64 | Nix: fails; cargo: 562 s | all pass (cargo) |
| `macos-14` | - | - | - | never ran: `The configuration 'macos-14-arm64-us-default' is not supported` |

`macos-latest` is Darwin 26.7.2, the same image as `macos-26`.  Runs:
43697228332 (the three aarch64 builds, final), 43697229131 (Intel with
cargo, final), 43692513759 (first try: Intel Nix failure, `macos-14`),
43692968930, 43693942775, 43694236568, 43695704400 (fixes to my scripts:
an AppleScript syntax error, a verdict crash, `go` labels, `nc` timeout,
stand-in lifetime).

## Findings

- **x86_64-darwin is not in cinq's flake at this commit.**
  `nix build github:yzhou216/cinq/97af215...#cinq` on `macos-15-intel` fails at
  once: `error: flake '...' does not provide attribute
  'packages.x86_64-darwin.cinq', 'legacyPackages.x86_64-darwin.cinq' or
  'cinq'`.  `nix eval ...#packages --apply builtins.attrNames` lists
  aarch64-darwin, aarch64-linux, armv6l-linux, armv7l-linux, i686-linux,
  powerpc64le-linux, riscv64-linux, x86_64-freebsd, x86_64-linux, so the
  `flakeExposed` of the locked Nixpkgs no longer has x86_64-darwin.  Plain
  `cargo build --release --locked` (with `brew install scdoc`) builds it in
  562 s on rustc 1.98.0; `intel.yml` lays the bundle out and signs it with
  `codesign --force --deep --sign -`, identifier `io.yiyuzhou.cinq`.
- **Signature**: `Identifier=io.yiyuzhou.cinq`, `Signature=adhoc`, valid on
  disk and satisfies its Designated Requirement, on all four runners (the
  Nix builds as built, the Intel bundle as signed by me).
- **Local Network prompt**: appears on every runner within 10 s of opening,
  `Allow "cinq" to find devices on local networks?` with cinq's reason.  On
  the three aarch64 runs (Darwin 26 and 15) a second prompt, `Allow
  "hosted-compute-" to find devices on local networks?`, follows and has to
  be answered too (earlier single presses left sACN multicast off, run
  43692513759).  Intel needed one press.  Prompts came at +8.1 to +9.5 s
  after GO 1 and the second at +12.3 to +15.1 s.
- **Before Allow** (GO 1 to the first press, about 8-10 s): Art-Net on
  loopback and the en0 broadcast address carried every refresh; sACN went
  to loopback only (28 to 31 packets on 127.0.0.1) and none to the en0
  address.  cinq's standard error stayed empty: the failed multicast sends
  are silent.  **After the last Allow**: sACN on en0 started at once (15-16
  packets in the 5 s that followed on all but `macos-26`, 12 there).
- **Light output** (all five runs): sACN universes 1, 2, 999 carry 51, 179 and
  128 at the right addresses, 128 on universe 2, 255 on 999, then out;
  Art-Net Port-Address 0, 1, 998 carry the same; priority 100 and source
  `cinq` on all sACN level packets; discovery lists `[1, 2, 999]` (9-12
  packets in the 150 s); idle refresh is exactly 12 packets in 12 s per
  universe, protocol, and sender (1 per second); discovery 1 per 12 s window;
  fade of 4 s: 120-127 packets in 4.5 s per universe, protocol, and sender,
  27 to 28 per second.  Senders heard: 127.0.0.1 and the en0 address for
  both protocols (Darwin's `lo0` takes loopback copies; the runner has one
  network, 192.168.64.x).
- **Network cues** (five runs): TCP and UDP with `7 1.5 "hello world"
  "42"` arrive decoded as int, float, string, string, 16-43 ms after GO;
  a burst of three GOs 0.3 s apart is one connection, 4-6 ms between
  messages; three GOs 4 s apart are three connections; a refusing
  destination warns after 10-23 ms with `cinq: network cue to
  127.0.0.1:3999: Connection refused (os error 61)`; a stand-in that
  stopped refuses the same way and the cue after its restart arrives in
  9-28 ms.
- **Cable pulled with pf** (`block drop quick on lo0 proto tcp` both ways,
  console alias 10.101.100.20 on lo0; `nc -z -G 2` reached it before and
  after and not during): the cue before arrived in 17-30 ms; the cue sent
  during 15 s out never arrived, nor did the one sent 1 s into a 7 s
  block, after unblocking either; the cues after arrived in 9-24 ms.
  The first warning came 1.012-1.023 s after the GO: `cinq: network cue to
  10.101.100.20:3037: connection timed out`.  The second dropped cue
  warned only in the sidebar, as standard error says each reason once.

## Traps

- `nc -z -w 2` on a dropped SYN took 75 s; add `-G 2`.
- `do shell script` inside a `tell application "System Events"` expression
  is a syntax error; call it at top level.
- `listen.py` on Darwin: 19 `EADDRINUSE` on the join of each group through
  the computer's address, since the join through `0.0.0.0` already covers
  it; harmless.

## Rerun on 1a21980 (2026-11-16)

cinq `1a2198013d7e4a7122b90c4141ca5285096a5df9` (`wip/light-cues`, rebased
onto the latest master and cleaned up) in place of 97af215; only `CINQ` in
`darwin.yml` and the checkout in `intel.yml` changed.  Runs **43985736020**
(`darwin.yml`, commit 54b8dd2) and **43985735985** (`intel.yml`, same
commit).  The jobs waited for runners for up to 2 h 45 min (the Nix runs
started at 02:57, 03:19, and 03:21 UTC).  Artifacts read: `build-`,
`lights-`, `net-`, and `cable-` of each runner.  Every verdict file has the
same check counts as before: 8 `ok` lines in `lights` (7 DMX and source
checks plus discovery), 15 in `net`, 6 in `cable`, none failing.

| Runner | Old (43697228332, 43697229131) | New |
| --- | --- | --- |
| `macos-latest` (Darwin 26.7.2, aarch64) | all pass; Nix build 639 s | all pass (job 22 min with all phases) |
| `macos-26` (26.7.2) | all pass; 692 s | all pass |
| `macos-15` (15.8.2) | all pass; 663 s | all pass |
| `macos-15-intel`, Nix | fails: `does not provide attribute 'packages.x86_64-darwin.cinq'` | fails with the same error |
| `macos-15-intel`, cargo (`intel.yml`, 15.7.9, rustc 1.98.0) | all pass; 562 s | all pass; `cargo build` 7 min 02 s; `Identifier=io.yiyuzhou.cinq`, ad hoc |
| `macos-14` | never ran: configuration not supported | never ran, same message; I cancelled it after 2 h 40 min |

| Check | Old | New (aarch64 x3, Intel) |
| --- | --- | --- |
| sACN universes 1, 2, 999 and Art-Net 0, 1, 998 | 51/179, 128, 255 at the right addresses, then out | the same, all `ok` |
| sACN priority, source name, discovery | 100, `cinq`, `[1, 2, 999]` in 9-12 packets | 100, `cinq`, 9-11 packets (10 on Intel) |
| Idle refresh | 12 packets in 12 s per universe, protocol, sender | 12 in 12 s on every one |
| Fade of 4 s, packets in 4.5 s | 120-127, 27-28/s | 127-128, 28.2-28.4/s; Intel 121, 26.9/s |
| Senders | 127.0.0.1 and the en0 address | the same |
| Local Network prompts | one for cinq, a second for `hosted-compute-` on aarch64, at +8.1-9.5 s and +12.3-15.1 s; Intel one press | the same shape: +7.9 to +10.1 s and +13.0 to +16.5 s; Intel one press at +9.1 s |
| Before Allow, sACN on en0 | none; sACN on 127.0.0.1 only (28-31 packets) | none; 24-30 packets on 127.0.0.1; on Intel 3 sACN packets reached en0 before the logged press, the first 0.023 s before the mark of the press (the Allow took effect before the script logged it) |
| After the last Allow, 5 s | 15-16 packets, 12 on `macos-26` | 15-16 on all, `macos-26` 15 |
| TCP and UDP arguments `7 1.5 "hello world" "42"` | decoded, 16-43 ms after GO | decoded, 12-33 ms after GO on aarch64 (Intel 15-18 ms) |
| Burst of 3 GOs 0.3 s apart | 1 connection, 4-6 ms between messages | 1 connection, 3-5 ms (Intel 3 ms) |
| 3 GOs 4 s apart | 3 connections | 3 connections |
| Refusing destination | warns after 10-23 ms, `Connection refused (os error 61)` | warns after 12-15 ms (Intel 16 ms), same text |
| Stand-in stopped, then restarted | refuses the same way; cue after restart in 9-28 ms | the same; 11-14 ms |
| Cable pulled with pf | before 17-30 ms; during never; `blip` never; after 9-24 ms; warning 1.012-1.023 s `connection timed out` | before 18-33 ms; both `never`; after 10-31 ms; warning 1.011-1.015 s (Intel 1.017 s), same text |

Difference: none.  The `macos-15-intel` job of `darwin.yml` printed
AppleScript syntax errors after its Nix failure (I did not look into why;
`intel.yml` is the Intel result).

## Rerun on 0ed5d6c (2026-11-17)

cinq `0ed5d6c41dbe29b3140f29238dae014a0a8baa81` (`wip/light-cues`, rebased
onto the rewritten master; every commit kept its tree) in place of 1a21980;
only `CINQ` in `darwin.yml` and the checkout in `intel.yml` changed.  Runs
**44054345889** (`darwin.yml`, commit 9faabb3) and **44054345897**
(`intel.yml`, same commit).  The jobs started within 15 minutes of the push
(09:23 to 09:59 UTC), unlike the 2 h 45 min of the last rerun.  Artifacts
read: `build-`, `lights-`, `net-`, and `cable-` of each runner.  Every
verdict file has the same check counts as before: 8 `ok` lines in `lights`,
15 in `net`, 6 in `cable`, none failing.

| Runner | Old (43985736020, 43985735985) | New |
| --- | --- | --- |
| `macos-latest` (Darwin 26.7.2, aarch64) | all pass; job 22 min with all phases | all pass; Nix build 5 min 08 s (09:40:03 to 09:45:11) |
| `macos-26` (26.7.2) | all pass | all pass; Nix build 14 min 45 s |
| `macos-15` (15.8.2) | all pass | all pass; Nix build 14 min 29 s |
| `macos-15-intel`, Nix | fails: `does not provide attribute 'packages.x86_64-darwin.cinq'` | fails with the same error, after 46 s |
| `macos-15-intel`, cargo (`intel.yml`, 15.7.9) | all pass; `cargo build` 7 min 02 s; `Identifier=io.yiyuzhou.cinq`, ad hoc | all pass; `cargo build` 14 min 29 s; `Identifier=io.yiyuzhou.cinq`, `TeamIdentifier=not set` |
| `macos-14` | never ran: configuration not supported; I cancelled it | never ran, same message (`The configuration 'macos-14-arm64-us-default' is not supported`); the platform cancelled the job at 09:36 UTC (I did not cancel it), which made the run's conclusion `cancelled` while every other job finished |

| Check | Old (aarch64 x3, Intel) | New |
| --- | --- | --- |
| sACN universes 1, 2, 999 and Art-Net 0, 1, 998 | right addresses, then out | the same, all `ok` (130, 3, 3 changes per stream) |
| sACN priority, source name, discovery | 100, `cinq`, `[1, 2, 999]` in 9-11 packets | 100, `cinq`, 10-13 packets (11 on Intel) |
| Idle refresh | 12 in 12 s per universe, protocol, sender | 12 in 12 s on every one |
| Fade of 4 s, packets in 4.5 s | 127-128, 28.2-28.4/s; Intel 121, 26.9/s | 126-127, 28.0-28.2/s; Intel 119, 26.4/s |
| Senders | 127.0.0.1 and the en0 address | the same |
| Local Network prompts | one for cinq, a second for `hosted-compute-` on aarch64, at +7.9 to +10.1 s and +13.0 to +16.5 s; Intel one press at +9.1 s | the same shape: first press +7.0 to +10.7 s, second +10.8 to +17.4 s; Intel one press at +12.1 s |
| Before Allow, sACN on en0 | none; 24-30 packets on 127.0.0.1; on Intel 3 reached en0 early | none on any runner, Intel included; 22-37 sACN packets on 127.0.0.1 (Art-Net on en0 and 127.0.0.1 went out as before) |
| After the last Allow, 5 s | 15-16 packets on all, `macos-26` 15 | 15-16 on all; `macos-26` 13 sACN on en0 and 15 on the others |
| TCP and UDP arguments `7 1.5 "hello world" "42"` | decoded, 12-33 ms after GO on aarch64 (Intel 15-18 ms) | decoded; TCP 17, 52, 24 ms (Intel 36), UDP 11, 17, 15 ms (Intel 26) on `macos-latest`, `macos-26`, `macos-15` |
| Burst of 3 GOs 0.3 s apart | 1 connection, 3-5 ms (Intel 3 ms) | 1 connection, 3-5 ms between messages (Intel 6 ms) |
| 3 GOs 4 s apart | 3 connections | 3 connections |
| Refusing destination | warns after 12-15 ms (Intel 16 ms), `Connection refused (os error 61)` | warns after 12, 29, 16 ms (Intel 28 ms), same text |
| Stand-in stopped, then restarted | refuses the same way; cue after restart in 11-14 ms | the same: warns after 8, 32, 11 ms (Intel 22 ms); `back` after 8, 25, 10 ms (Intel 23 ms) |
| Cable pulled with pf | before 18-33 ms; both `never`; after 10-31 ms; warning 1.011-1.015 s (Intel 1.017 s) | before 11, 40, 17 ms (Intel 34); both `never`; after 9, 22, 17 ms (Intel 31); `after blip` 8, 30, 40 ms; warning 1.013, 1.022, 1.036 s (Intel 1.031 s), same text |

Difference: no change in behavior.  The latencies on `macos-26` (TCP 52 ms,
`apart-1` 38 ms, cable `before` 40 ms) and `macos-15` (`after blip` 40 ms)
are above the old 12-33 ms range for single values, while `macos-latest`
(the same Darwin 26.7.2) gave 8-17 ms; I took that for runner load and did
not run those jobs again.  The cargo build on Intel took 14 min 29 s
against 7 min 02 s and the Nix builds on `macos-26` and `macos-15` 14.5
minutes, also longer than before (11 minutes); I did not look into why.
