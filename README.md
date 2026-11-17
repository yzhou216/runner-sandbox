<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq 97af215 light and network cues on GNU/Linux, with OLA as a DMX gateway

Branch `cinq/2026-11-15-linux-lights`, workflow `.github/workflows/lights.yml`.
cinq is built natively with `nix build github:yzhou216/cinq/97af215a58520fe2a30ae5d2bffb98f90800a9e6#cinq`
on `ubuntu-latest` (x86_64, 10.1.0.x) and `ubuntu-24.04-arm` (aarch64), then
run in headless sway.  Each phase uploads an artifact `<arch>-<phase>` of its
own: `runner`, `light` (with `network` beside it), `stress`, `lan`,
`powercut`, `refuse`, `gateway`.

Runs: 43693183248 (all phases but `refuse` and the downstream part of
`gateway`, both arches green), 43696116461 (adds `refuse`; its attempt to read
OLA's Art-Net output failed, see below), **43698236045** (final, every job and
step green on both arches).  Both architectures gave the same results to the
digit unless a number is listed twice.

The scripts come from `cinq/2026-11-13-eos-offline` (`eos/linux.sh`,
`stress.sh`, `lan.sh`, `powercut.sh`, `listen.py`, `fakeeos.py`), with these
changes: `linux.sh` takes a show and a number of GOs and closes cinq's window
at the end; `listen.py` joins the groups on every network (`ip -4 addr`), takes
`LISTEN_UNIVERSES` and `LISTEN_SUMMARY` and reports sACN packets with the
stream terminated option.  New under `ola/`: `gateway.toml`, `check-light.py`,
`sacn.py` (the stand-in console), `gateway.py` and `gateway.sh` (OLA test),
`refuse.py` and `refuse.toml`.

## Light output (`linux.sh` with `ola/gateway.toml`, five GOs, `check-light.py`)

All 20 checks `ok` on both arches.

| Check | Result |
| --- | --- |
| DMX values, universes 1, 2, 999 | 77/51/255 (30/20/100 percent), 51/179 (20/70), 128 (50); a fade 77 to 179 and back; then 0 |
| sACN universe, Art-Net Port-Address | universe n is sACN n and Art-Net n-1 (999 is 998), 512 slots each |
| sACN source | name `cinq`, priority 100 only |
| Discovery | lists [1, 2, 999], every 10 s (3 packets in 35 s) |
| Idle refresh | 1 packet/s per universe, protocol, and network |
| Packets during a fade | 34-35/s per universe, protocol, and network in a 3 s fade; 21 in a 0.5 s fade |
| Networks that carried packets | 10.1.0.168 (aarch64: 10.1.0.14), 127.0.0.1, and 172.17.0.1 (docker0, `NO-CARRIER`), identical data on each |
| Before the first cue | zeros on all universes (OLA reads nothing), packets from the start |
| When cinq quits (window closed) | packets stop at once, no packet with the stream terminated option |

## Network cues

| Check | Result |
| --- | --- |
| TCP SLIP `/eos/cue/1/fire`, UDP `/test "hello world" 7 2.5 "12"` | arrive as `[]` and `['hello world', 7, 2.5, '12']` |
| 50 TCP cues in one GO, 50 UDP | 1 connection, all 50 and 50 in order |
| fakeeos killed, restarted before the next GO | 10 cues over a new connection, in order (3 connections in all) |
| soak: 300 cues 0.1 s apart | 300 in order; cinq RSS flat (58.6 MB x86_64, 71.3 MB aarch64) |
| GOs a few seconds apart | one connection each, closed 1 s later |
| Cable pulled (`lan.sh`, console at 10.101.100.20 behind a veth) | `before`, `after`, `after again`, `after blip` arrive less than 10 ms after GO; cue while out never arrives; **cue 1 s after the pull never arrives** (`blip: never`); sidebar and standard error `cinq: network cue to 10.101.100.20:3037: connection timed out` |
| Power cut under cinq (iptables) | GO after the cut and the next both arrive on new connections |
| ion-test.toml universe 999 | sACN and Art-Net 998 seen at the far end of the cable |
| Refused (`refuse.py`) | `cinq: network cue to 127.0.0.1:3999: Connection refused (os error 111)`, 0.00 s after the key |
| Name that does not resolve | `cinq: network cue to nonexistent.invalid:3037: failed to lookup address information: Name or service not known`, 0.00 s |
| No route answers (10.255.255.1) | `cinq: network cue to 10.255.255.1:3037: connection timed out`, 1.00 s |
| UDP to a closed port | no warning |
| Same refusal again | no second line on standard error (once per reason); sidebar `Warnings (3)` in `refuse/go-5.png` |

`Io error: Broken pipe (os error 32)` three times in `stress` and `powercut`
cinq logs is the script ending sway while cinq still runs.

## OLA 0.10.9 as the gateway (`ola/gateway.py`)

olad with every plugin off but E1.31 (4 input ports) and Art-Net, as in the
brief; the E1.31 device chose the default route address on its own.  Ports
patched with `ola_patch -d DEV -p PORT -u UNIVERSE -i`.  E1.31 universe n is
OLA universe n; an Art-Net input port at OLA universe n is Port-Address n
(`ArtNet Universe 0:0:n`).  Levels read over HTTP (`/get_dmx`, 40 times a
second) and with `ola_recorder`.

| Check | Result |
| --- | --- |
| (a) cinq universes 1, 2, 999 in OLA universes 1, 2, 999 | exact: {1:77, 3:51, 5:255}, {1:51, 2:179}, {1:128}; first level 0.03-0.04 s after the key |
| (b) stand-in console, sACN priority 100, universe 1: address 1 = 102, 3 = 128, 6 = 77, while cinq has 1 = 77, 3 = 51, 5 = 255 | OLA holds {1:102, 3:128, 5:255, 6:77}, merged 0.08 s after the console starts |
| (b) cinq fades address 1 from 77 to 179 over 3 s with the console on | stays 102 until cinq passes it, then follows to 179, never below 102, monotonic; addresses 3, 5, 6 constant in every frame; 102 to 179 took 2.19 s (cinq's fade says 2.26 s) |
| (c) console at priority 1 (1 = 200, 3 = 255, 7 = 99) while cinq sends at 100 | ignored whole: OLA holds only cinq's {1:179, 3:51, 5:255} |
| (c) cinq quits | the priority-1 console takes over 2.05 s after the process ends ({1:200, 3:255, 7:99}) |
| (d) cinq alone quits, window closed or killed | **OLA never drops its levels**: universe 1 still held 77/51/255 60 s after, no change at all; the same for a console that sends terminate packets and stops (held 15 s) |
| (e) 3 s fade 77 to 179 | 102 frames in 2.92 s, 34.6 frames/s, every value from 77 to 179 once, monotonic, largest step 1; 104 frames in 4.5 s of recording |
| (f) Art-Net input ports at OLA universes 0, 1 | cinq universe 1 arrives as Port-Address 0 ({1:77, 3:51, 5:255}) and universe 2 as Port-Address 1 ({1:51, 2:179}) |
| (g) OLA output port (dummy plugin, patched to universe 1) | frames: look 0.52 s after the key (the 0.5 s fade ends), 34.3 frames/s in the fade, monotonic, 3 in the 3.2 s between cues (cinq's refresh), last frame {1:179, 3:51, 5:255}; **0 frames in the 30 s after cinq quits** |

What this means: a gateway built on OLA keeps the last look when cinq ends or
crashes, and receivers of its output hold it too; only a source of lower
priority replaces it, after 2.05 s.  OLA's universe merge mode stays LTP in
`ola_uni_info`, but the E1.31 plugin merges its sources HTP per address, and
only within the highest priority.

Also seen: OLA's Art-Net input merges two sources per universe and logged
`Max merge sources reached, ignoring` 54 times in the Art-Net phase, since
cinq's Art-Net comes from three source addresses (eth0, 127.0.0.1, docker0).
The data of the three is identical, so nothing is lost.  Art-Net output of
OLA to a real node needs the node to answer ArtPoll (run 43696116461 heard
OLA's ArtPoll and ArtPollReply but no ArtDmx), so the dummy port stands for
the output.

## Not tested

- Multicast between two machines or through a switch; the runner is one
  machine, so sACN reached OLA through eth0's own multicast loop.
- The real Ion Xe, its per-address priority (start code 0xDD) packets, or a
  hardware DMX output.

## Rerun on 1a21980 (2026-11-16)

cinq `1a2198013d7e4a7122b90c4141ca5285096a5df9` (`wip/light-cues`, rebased
onto the latest master and cleaned up) in place of 97af215.  Only the commit
in `.github/workflows/lights.yml` changed.  Run **43985729726** (commit
4c08a91, both arches green, every step green; it waited 86 minutes for
runners before it started).  Artifacts read from it: `<arch>-light`,
`-stress`, `-lan`, `-powercut`, `-refuse`, `-gateway`, `-runner`.

| Check | Old (43698236045) | New (43985729726) |
| --- | --- | --- |
| Light output, 20 checks (`check-light.py`) | all ok, both arches | all ok, both arches; senders 10.1.0.211 (aarch64 10.1.0.67), 127.0.0.1, 172.17.0.1 |
| DMX values, universes 1, 2, 999 | exact | exact |
| sACN priority 100 only, source `cinq`, discovery [1, 2, 999] every 10 s, 512 slots | yes | yes, no packet with the stream terminated option after quitting |
| Idle refresh, fade rate | 1/s; 34-35/s, 21 in the 0.5 s fade | 1/s; 34 in the fade, 21 in the 0.5 s fade, identical on every sender and arch |
| Network cues: TCP, UDP arguments | `[]`, `['hello world', 7, 2.5, '12']` | the same |
| 50 TCP, 50 UDP in one GO | 1 connection, in order | 1 connection each way, all 50 and 50 |
| Console restarted before a GO | 10 cues, 3 connections in all | 10 cues, 3 connections |
| Soak, 300 cues 0.1 s apart | 300, RSS flat 58.6 MB / 71.3 MB | 300, RSS flat 58.9 MB / 71.5 MB |
| Cable pulled (`lan.sh`) | before, after, again, blip arrive under 10 ms; while out and `blip` never; warning `connection timed out` | the same: -8 ms (x86_64), -5 ms (aarch64) after GO; both `never`; same warning |
| Power cut | GO after the cut and the next arrive on new connections | `/cut 2` and `/cut 3` on new connections |
| ion-test.toml, universe 999 | sACN and Art-Net 998 at the far end | 155 changes each, sources `cinq` priority 100 |
| Refused, no resolve, no route, repeat | os error 111 at 0.00 s; lookup failure 0.00 s; timed out 1.00 s; no second line; `Warnings (3)` | the same text and times (0.00, 0.00, 1.00 s); `Warnings (3)` in `refuse/go-5.png` |
| OLA (a) to (g) | exact, merge 0.08 s, 2.05 s takeover, never drops, 34.6 frames/s | all `ok`; first level 0.03 s, merge 0.06 s, takeover 2.05 s, never drops (60 s), 34.7 frames/s (aarch64 34.9), fade 102 to 179 in 2.17 s, dummy port 34.3/s, 0 frames after quit |

Difference: none beyond the numbers above (merge 0.08 to 0.06 s, fade rate
34.6 to 34.7 frames/s).  The two OLA phases printed the same numbers on
x86_64 and aarch64.
