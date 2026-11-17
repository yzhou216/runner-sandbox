<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq's light and network cues on riscv64 and FreeBSD

Tests cinq 97af215 (`wip/light-cues`) on riscv64 GNU/Linux (RISE's
`ubuntu-24.04-riscv`, a `scaleway-em-rv1` board, Linux 5.10.113) and on
x86_64 and aarch64 FreeBSD 15.1-RELEASE-p6 (`vmactions/freebsd-vm@main` on
`ubuntu-latest`; aarch64 is emulated).

## What runs

`.github/workflows/lights.yml`, with the scripts in `eos/` (the listener,
stand-in console, and OSC code come from `cinq/2026-11-13-eos-offline`):

- **src** fetches the source of the commit.  **cross** builds the closures of
  `#riscv64-linux`, `#x86_64-freebsd`, and `#aarch64-freebsd` with Nix on
  `ubuntu-latest`, 6 to 10 minutes each with the Cachix cache `yzhou216`.
- **riscv-unit**, **freebsd-unit**: `unit.sh` builds the tests of cinq with
  `CINQ_SHARE_DIR=` empty, runs `cargo test network`, `light`, and `osc`,
  and then each matching test alone with its time.  Rust and the libraries
  come from apt (RISE, with `liboss4-salsa-asound2-` to swap ALSA's library)
  and `pkg`.
- **riscv-gui**, **freebsd-gui**: `play.sh` runs `lights.toml` in the
  cross-built cinq in headless sway, with one GO.  `listen.py` records
  sACN and Art-Net as JSON and a pcap, and `fakeeos.py` stands in for a
  console that goes down for 2.7 s and comes back.  `analyze.py` reads what
  they wrote; its output is in `results/gui-*.txt`.  `lights.toml` chains
  its cues: lights on universes 1, 2, and 999 (20, 50, 70, 100 percent, and
  a 4 s fade), then network cues (TCP and UDP with int, float, and quoted
  arguments, a refused port, a burst of five within half a second, three
  cues 3 s apart, one while the console is down and two after, and a
  destination that never answers with a cue behind it).

## Results

| Platform | Unit tests (debug build) | Show |
| --- | --- | --- |
| riscv64 GNU/Linux | 34 of 34 passed; build 4837 s | played, all values exact |
| x86_64 FreeBSD | 34 of 34 passed; build 535 s | played, all values exact |
| aarch64 FreeBSD, emulated | 34 of 34 passed; build 14979 s | played, all values exact; fade sent 10.6 packets/s |

Unit tests (suite times as libtest reports them; per-test times in
`results/unit-*-tests.tsv`):

| Platform | network (11) | light (18) | osc (5) | Slowest test |
| --- | --- | --- | --- | --- |
| riscv64 | 1.57 s | 0.02 s | 0.01 s | quiet second, new connection: 1.56 s |
| x86_64 FreeBSD | 1.54 s | 0.00 s | 0.00 s | the same: 1.54 s |
| aarch64 FreeBSD | 1.63 s | 0.13 s | 0.02 s | not timed alone |

Every platform gave the same output of the show:

- DMX: 20 percent as 51 (universe 1, address 1), 50 as 128 (universe 2,
  address 10), 100 as 255 (universe 999, address 512), 70 as 179 and 100 as
  255 (addresses 1 and 2), the fade to 0, 77 (30 percent), and 204 (80
  percent).  Art-Net Port-Address 0, 1, and 998 carried universes 1, 2, and
  999.  sACN had priority 100 and the source name `cinq`.
- Discovery listed `[1, 2, 999]` every 10.0 s (4.98, 14.98, ... on x86_64
  FreeBSD).  Idle refresh was about one packet a second per universe.
- Packets a second per universe and sender during the 4 s fade: 37.5 to
  37.8 on riscv64, 38.3 on x86_64 FreeBSD, 10.6 on aarch64 FreeBSD, whose
  emulated processor was the limit (cinq used 136 percent of a core).
- TCP and UDP messages arrived as `/test [7, 2.5, '12', 'hello']`.  The
  burst of five went over one connection, about 0.1 s apart; the three cues
  3 s apart each made a connection, which the console saw close 1.0 s after
  the message.
- A refused port warned within 0.1 s of its cue: `cinq: network cue to
  127.0.0.1:3999: Connection refused (os error 111)` on GNU/Linux, `(os
  error 61)` on FreeBSD.  The cue while the console was down warned the same
  way for 127.0.0.1:3037 and was not sent again; the two cues after it
  arrived at once.
- A destination that never answers (10.255.255.1:3037) warned
  `connection timed out` 1.00 s after its cue on riscv64 (1.03 s on
  FreeBSD).  The cue behind it, to the console, arrived at that moment, 0.8
  s after its own cue.

Networks: riscv64 had `lo`, `end0` (62.210.163.60/27), `flannel.1` (a /32),
`cni0`, and `docker0` (no carrier); Art-Net was heard from all five
addresses, 780 packets each, and sACN from `lo` and `end0` alone, which is
only what `listen.py` joined (the default network, and loopback by unicast).
FreeBSD had `vtnet0` (192.168.122.10) and `lo0`; both protocols arrived from
both, 831 and 837 packets each on x86_64.

## Suspected defects

- **The window of cinq on FreeBSD has no app ID.**  `src/front.rs:75`
  sets `application_id` only under `target_os = "linux"`, and sway's tree on
  FreeBSD reads `"name": "lights.toml - cinq"` with `"app_id": null`
  (run 43696029330, artifact `gui-x86_64-freebsd`, `tree-before.json`).
  Window rules by app ID cannot match there, which the comment at that line
  says matters.  iced's `PlatformSpecific` has no such field outside GNU/Linux,
  so this may need iced.  `play.sh` finds the window by its title as well.
- **cinq used a whole core with ICED_BACKEND=tiny-skia in a 1600x1000
  headless sway:** 54.9 s of processor time in about 52 s on x86_64 FreeBSD
  (`results/cpu-x86_64-freebsd.txt`), 100.5 s in about 57 s on aarch64.  Not
  measured on riscv64.  Probably the cost of drawing in software on every
  tick, but worth knowing.
- When the compositor ends, cinq prints `Io error: Broken pipe (os error
  32)` three times (all runs).

## Runs

| Run | What |
| --- | --- |
| 43693040726 | Everything.  riscv64 unit and show passed; FreeBSD unit passed (aarch64 after 4 h 18 min); FreeBSD shows ran but their output was not uploaded |
| 43694493043 | FreeBSD uploads added: x86_64 unit with times; shows waited 300 s for a window that has no app ID |
| 43696029330 | The state of sway when the window is missing: it showed the title and a null app ID |
| 43697377734 | FreeBSD shows, window found by title: both played |

## Not tested

- Per-test times on aarch64 FreeBSD, and the shows' CPU use on riscv64:
  those runs came before the scripts wrote them.
- Which networks carry sACN on riscv64 beyond `end0`: `listen.py` joins no
  group on the others.
- Sound, video, and the Local Network prompts: not part of this task.
- The riscv64 show used Nixpkgs' glibc, as the closure brings it; a build
  from Ubuntu's was not tried.

## Traps

- FreeBSD's sway lists a window without an app ID as `null`; match the title.
- `vmactions/freebsd-vm` copies the workspace back but never uploads it;
  tar it and upload in a later step.
- A `cargo test --no-run` of cinq takes 4837 s on RISE, 535 s on x86_64
  FreeBSD, and 14979 s on emulated aarch64 FreeBSD.

## Rerun on 1a21980 (2026-11-16)

cinq `1a2198013d7e4a7122b90c4141ca5285096a5df9` (`wip/light-cues`, rebased
onto the latest master and cleaned up) in place of 97af215.  Only the commit
in `lights.yml` changed.  Run **43985744045** (commit 9f1b2b3, message tag
`[skip aarch64 unit]`, so the unit tests ran on riscv64 and x86_64 FreeBSD but
not on emulated aarch64 FreeBSD, which took 4 h 18 min before): all nine jobs
green.  The jobs waited for runners for 1.5 hours.  Artifacts read:
`unit-riscv64`, `unit-x86_64-freebsd`, `gui-riscv64`, `gui-x86_64-freebsd`,
`gui-aarch64-freebsd`; `analyze.py` ran over the three `gui-*` ones.

| Platform | Old (43693040726, 43697377734) | New (43985744045) |
| --- | --- | --- |
| cross builds of `#riscv64-linux`, `#x86_64-freebsd`, `#aarch64-freebsd` | built | built, all three |
| riscv64 unit, `cargo test --no-run` | 4837 s | 6001 s on another board (`riscv-runner-52`; the suite grew from 400 to 858 tests) |
| riscv64 unit, network (11), light, osc (5) | 11, 18, 5 passed; 1.57 s, 0.02 s, 0.01 s | 11, 26, 5 passed; 1.59 s, 0.07 s, 0.01 s; 42 of 42 alone, none failing; slowest `tcp_message_after_a_quiet_second...` 1.58 s (old 1.56 s) |
| x86_64 FreeBSD unit | 535 s; 11, 18, 5; 1.54 s, 0.00 s, 0.00 s; 34 of 34 | 538 s; 11, 26, 5; 1.54 s, 0.01 s, 0.00 s; 42 of 42; slowest 1.54 s |
| aarch64 FreeBSD unit | 34 of 34 passed; 14979 s | skipped as instructed |
| riscv64 show | played, all values exact | played, all values exact |
| x86_64 FreeBSD show | played, all values exact | played, all values exact |
| aarch64 FreeBSD show | played, all values exact | played, all values exact |
| DMX (all three shows) | 51 (u1 address 1), 128 (u2, 10), 255 (u999, 512), 179 and 255, fade to 0, 77, 204; Art-Net 0, 1, 998; priority 100, `cinq` | identical final states and holds (e.g. `{1: 51}` 4.0 s, `{1: 179, 2: 255}` 3.0 s) |
| Fade rate per universe and sender | 37.5-37.8/s riscv64; 38.3/s x86_64 FreeBSD; 10.6/s aarch64 FreeBSD | 37.8/s; 38.6/s; 16.9/s (emulated; cinq 130 percent of a core, old 136) |
| Idle refresh, discovery | about 1/s per universe; `[1, 2, 999]` every 10.0 s | the same (0.7 to 1.0/s in the windows, discovery at 4.98, 14.98, ... on x86_64) |
| Packets per sender | x86_64 FreeBSD 831 Art-Net, 837 sACN; riscv64 780 and 786 from five addresses | x86_64 FreeBSD 831 and 837 (the same); riscv64 795 and 801 from seven addresses (`flannel`, `docker0`, and two new Docker bridges) |
| TCP and UDP `/test [7, 2.5, '12', 'hello']` | arrived | arrived |
| Burst of five | one connection, about 0.1 s apart | the same (0.11 s steps on riscv64) |
| Three cues 3 s apart | each a connection, closed 1.0 s after | the same |
| Refused port | `Connection refused (os error 111)` / `(os error 61)` on FreeBSD, within 0.1 s | the same text, at t+16.19 to 16.40 s as before (within 0.4 s of the old times) |
| Cue while the console is down | warned once; the two cues after arrived at once | the same |
| No answer, 10.255.255.1:3037 | `connection timed out`, 1.00 s riscv64, 1.03 s FreeBSD; the cue behind it arrived 0.8 s after its own cue | `connection timed out` at t+41.51 to 41.82 s (old 41.28 to 41.81); the cue behind it arrived (`/behind` at t+41.57 on riscv64) |
| Compositor ends | `Io error: Broken pipe (os error 32)` three times | the same |
| cinq CPU with `tiny-skia` on x86_64 FreeBSD | 54.9 s in about 52 s | 54.9 s |
| Window of cinq on FreeBSD | title only, `app_id` null | found by title as before (not re-examined) |

Difference: none in behavior.  Bigger numbers come from the code under test
(the `light` filter now matches 26 tests, 8 more than before, since master added names such as
`highlight_follows...` and `the_sidebar_lights_only_while...`, none in `back::light`, whose 3 tests are unchanged), the riscv64
board, and emulation speed on aarch64.

## Rerun on 0ed5d6c (2026-11-17)

cinq `0ed5d6c41dbe29b3140f29238dae014a0a8baa81` (`wip/light-cues`, rebased
onto the rewritten master; every commit kept its tree) in place of 1a21980.
Only the commit in `lights.yml` changed.  Run **44054355789** (commit
3c72884, message tag `[skip aarch64 unit]`): all nine jobs green (09:33 to
10:14 UTC; no long wait for runners this time).  Artifacts read:
`unit-riscv64`, `unit-x86_64-freebsd`, `gui-riscv64`, `gui-x86_64-freebsd`,
`gui-aarch64-freebsd`; `analyze.py` ran over the three `gui-*` ones.

| Platform | Old (43985744045) | New (44054355789) |
| --- | --- | --- |
| cross builds of `#riscv64-linux`, `#x86_64-freebsd`, `#aarch64-freebsd` | built, all three | built, all three |
| riscv64 unit, `cargo test --no-run` | 6001 s on `riscv-runner-52` | 238 s on `riscv-runner-47` (kernel 5.10.113-scw1; I did not look into the difference) |
| riscv64 unit, network (11), light, osc (5) | 11, 26, 5 passed; 1.59 s, 0.07 s, 0.01 s; 42 of 42 alone | 11, 26, 5 passed; 1.58 s, 0.05 s, 0.01 s; 42 of 42 alone, none failing |
| x86_64 FreeBSD unit | 538 s; 11, 26, 5; 1.54 s, 0.01 s, 0.00 s; 42 of 42 | 531 s; 11, 26, 5; 1.54 s, 0.01 s, 0.00 s; 42 of 42 |
| aarch64 FreeBSD unit | skipped as instructed | skipped as instructed |
| riscv64 show | played, all values exact | played, all values exact |
| x86_64 FreeBSD show | played, all values exact | played, all values exact |
| aarch64 FreeBSD show | played, all values exact | played, all values exact |
| DMX (all three shows) | `{1: 51}` 4.0 s, `{1: 179, 2: 255}` 3.0 s, u2 `{10: 128}` 8 s, u999 `{512: 255}`, finals `{2: 77}`, `{10: 204}`; Art-Net 0, 1, 998; priority 100, `cinq` | identical final states and holds on all three (x86_64: `{1: 51}` 4.0 s from t+0.99, `{1: 179, 2: 255}` 3.0 s from t+5.99, `{10: 128}` 8.0 s, `{512: 255}` 45.0 s, then `{2: 77}` and `{10: 204}` for 33.0 s) |
| Fade rate per universe and sender | 37.8/s riscv64; 38.6/s x86_64 FreeBSD; 16.9/s aarch64 FreeBSD | 35.3/s; 38.3/s; 15.3/s (emulated; cinq 132.5 percent of a core, old 130) |
| Idle refresh, discovery | 0.7 to 1.0/s; `[1, 2, 999]` every 10.0 s | 0.8/s (aarch64 1.2/s) in the idle window, 1.0/s between cues; discovery at 4.99, 15.96, 25.97 ... on x86_64 FreeBSD |
| Packets per sender | x86_64 FreeBSD 831 Art-Net, 837 sACN; riscv64 795 and 801 from seven addresses | x86_64 FreeBSD 831 and 837 (the same); riscv64 762 and 774 from seven addresses (127.0.0.1, 172.17.0.1, 172.18.0.1, 172.19.0.1, 10.244.65.0, 10.244.65.1, 62.210.163.219); aarch64 FreeBSD 408 and 414 |
| TCP and UDP `/test [7, 2.5, '12', 'hello']` | arrived | arrived on all three, UDP 0.2 s after TCP |
| Burst of five | one connection, 0.11 s steps on riscv64 | one connection, 0.09 to 0.11 s steps on all three |
| Three cues 3 s apart | each a connection, closed 1.0 s after | the same (3 connections, 3.0 s apart) |
| Refused port | `Connection refused (os error 111)` / `(os error 61)` on FreeBSD, at t+16.19 to 16.40 s | the same text, at t+16.15 (riscv64), 16.40 (x86_64 FreeBSD), 15.91 s |
| Cue while the console is down | warned once; the two cues after arrived at once | the same: warned once at t+31.57 to 31.80 s (31.29 s aarch64), `/back` and `/after` arrived |
| No answer, 10.255.255.1:3037 | `connection timed out` at t+41.51 to 41.82 s; `/behind` arrived | `connection timed out` at t+41.55 (riscv64), 41.78 (x86_64), 41.27 s (aarch64); `/behind` arrived at the same time |
| Compositor ends | `Io error: Broken pipe (os error 32)` three times | the same |
| cinq CPU with `tiny-skia` on x86_64 FreeBSD | 54.9 s | 54.1 s (100.5 percent of a core) |
| Window of cinq on FreeBSD | found by title as before (not re-examined) | the same (not re-examined) |

Difference: none in behavior.  The riscv64 fade ran at 35.3 frames/s against
37.8 (on a different board again; I did not look for the cause), and aarch64 FreeBSD at 15.3 against 16.9 (emulated); the
x86_64 FreeBSD numbers equal the earlier ones.  The riscv64 build took 238 s
against 6001 s on the other board.
