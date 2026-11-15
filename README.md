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
