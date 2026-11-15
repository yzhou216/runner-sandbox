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
