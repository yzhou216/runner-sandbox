<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq's light and network cues on Windows, x86_64 and aarch64

Tests of cinq 97af215a58520fe2a30ae5d2bffb98f90800a9e6 (branch
`wip/light-cues`) on `windows-latest` (Windows Server 2025 Datacenter
10.0.26100, x86_64) and `windows-11-arm` (Windows 11 Enterprise, aarch64).
cinq is cross-built on `ubuntu-latest` with `nix build
github:yzhou216/cinq/97af215...#x86_64-windows` and `#aarch64-windows` and
handed to the Windows jobs as artifacts.  The PE machine of the two
`cinq.exe` is 0x8664 and 0xAA64.  Everything runs with `ICED_BACKEND=tiny-skia`
and GO is a SendKeys space into the window of cinq.

## What runs

`.github/workflows/windows.yml`, scripts in `eos/`:

- **build** (two jobs): the cross builds, 5 to 7 minutes each.
- **cues**: `test.ps1` starts cinq with `lights.toml` (instruments on
  universes 1, 2, 999) beside `listen.py` (sACN and Art-Net on the same
  computer, JSON lines and a pcap), presses GO three times (a preset of 20, 30,
  40, 50, 70, 80, 100 percent; a fade of four seconds; a blackout), then starts
  `fakeeos.py` and cinq with `net.toml` and presses GO for 24 network cues.
  Marks in seconds since the epoch line up with the pcap and the stand-in's
  log.  `analyze.py DIR` reads the pcap.  The firewall is on.
- **eos** (`windows-11-arm`): ETC Eos 3.3.10.28, an x86_64 program, under
  emulation, with aarch64 cinq beside it; the scripts of branch
  `cinq/2026-11-13-eos-offline`.
- **firewall**: `firewall.ps1` makes a copy of python.exe listen and tries to
  answer the firewall's prompt.

`fakeeos.py` is patched against the earlier branch for Windows: it passed
`reuse_port=True` (Windows has no SO_REUSEPORT) and did not catch a reset
connection.  `test.ps1` also adds an allow rule for python.exe, since the
firewall's prompt on `windows-11-arm` takes the keys (below), and starts cinq
before the listener, so that a prompt of cinq's own would show.

## Runs

| Run | What | Outcome |
| --- | --- | --- |
| 43692189824 | first run of everything | x86_64 passed; aarch64 lights and network cues never fired (the prompt of the firewall took every GO); Eos on aarch64 passed |
| 43693917753 | everything, with python allowed and cinq first | all passed on both architectures; Eos on aarch64 passed again |
| 43695978974 | firewall probe | listener never started (quoting); nothing learned |
| 43696223664 | firewall probe, listener in a file | prompt on `windows-11-arm` only; clicking Allow answered it |

## Light output (test 1), run 43693917753 unless noted

| Check | x86_64 | aarch64 |
| --- | --- | --- |
| DMX after GO 1 (percent -> value) | u1: slots 1-5 and 512 = 51, 77, 102, 128, 179, 204; u2 slot 10 = 255; u999 slots 1, 512 = 51, 255 | same |
| Art-Net Port-Address | 0, 1, 998 for universes 1, 2, 999 | same |
| After the fade | u1 slot 1 = 255, slot 2 = 0; u2 slot 10 = 128; u999 slot 512 = 0 | same |
| After the blackout | every slot 0 | same |
| sACN priority, source name, length | 100, `cinq`, 512 slots | same |
| Idle refresh per stream | 1.05 to 1.07 packets/s | 1.03 to 1.07 |
| Packets in a fade of 4 s | 126 in 3.8 s, 33.2/s, median gap 30 ms, longest 340 ms (before the first step) | 120 in 3.8 s, 31.6/s, median 32 ms, longest 318 ms |
| Fade steps of slot 1 (sACN) | 53 at 0.05 s to 249 at 3.88 s, smooth | 53 to 247, smooth |
| Discovery (universe 64214) | lists (1, 2, 999), every 10.0 to 10.4 s per source | every 10.1 to 10.6 s |
| After cinq ends | no packet from 1 to 12 s | none |
| Before the first cue | cinq already refreshes zeros once a second | same |

Interfaces: each universe went out as sACN and as Art-Net from three source
addresses with equal counts (189 packets of each stream in run 43692189824):
the Ethernet address (10.1.0.x), the Hyper-V "vEthernet (Default Switch)"
address (172.x.x.1, a different range each run), and 127.0.0.1, the
loopback copy.  On aarch64 the network list was the same three plus a
Mellanox virtual adapter that has no IPv4 address.  The listener joined the
sACN groups on 0.0.0.0; joins on the Ethernet address itself failed with
`[WinError 10022]` (19 joins on both architectures, `dmx.jsonl`), which did
not stop any packets from arriving.

## Network cues (test 2), run 43693917753

All 24 cues behaved the same on both architectures.

| Check | Result |
| --- | --- |
| TCP and UDP arguments `"hello world" 7 2.5 "12" -4 1e3 abc` | decoded as `['hello world', 7, 2.5, '12', -4, 1000.0, 'abc']` over both |
| Latency from GO to arrival on the stand-in | x86_64 3 to 8 ms, aarch64 9 to 12 ms (cues that arrive) |
| Burst of 4 cues with `advance = "continue"` | one connection, four messages in the same millisecond, closed 1.0 s after the last |
| Two GOs 4 s apart | one connection each (distinct ports) |
| Two GOs 0.4 s apart | one connection, messages 0.41 s apart |
| `localhost:3037` | arrived (resolves to 127.0.0.1) |
| Refused, nothing listens on 127.0.0.1:3999 | `cinq: network cue to 127.0.0.1:3999: connection timed out`, 0.911 s (x86_64) and 0.946 s (aarch64) after GO; the next cue arrived |
| No answer, 203.0.113.7:3037 | `cinq: network cue to 203.0.113.7:3037: connection timed out`, 1.024 s and 0.987 s after GO |
| 255.255.255.255, 224.0.0.1, 0.0.0.0 with port 3037 | `The requested address is not valid in its context. (os error 10049)` each, written before my poll began, so within about 50 ms |
| Name that does not resolve | `cinq: network cue to nonexistent.invalid:3037: No such host is known. (os error 11001)`, 0.03 s |
| Stand-in killed and restarted 4 s later | the next GO made a new connection and arrived at once |
| UDP to a port nobody listens on | no warning, nothing arrives |

On Windows a closed port on the same computer never shows as refused: cinq's
one second limit expires first and the warning says `connection timed out`.
No real "unreachable" (a missing route) could be made without cutting the
runner off, so the closest were the invalid addresses above and the
silent 203.0.113.7.  The warning in the sidebar could not be photographed:
cinq's window is 1456 pixels wide on a 1024 pixel desktop.

## Platform specifics

- **Firewall.**  cinq opens no listening socket and created no firewall rule
  or prompt on either Windows (screenshots `windows-lights-cinq-alone`,
  `firewall-new-rules.txt`).  On `windows-latest` (a server) even a program
  that listens gets no prompt and no rule (`firewall-windows-latest`).  On
  `windows-11-arm` a listening program, `listen.py` or `fakeeos.py`, brings up
  "Windows Security: Do you want to allow public and private networks to
  access this app?" at once, and Windows adds two inbound Block rules for it
  (profile Public) before anyone answers.  The prompt is a top-level window
  that UI Automation does not list (`EnumWindows` finds it, titled
  `Windows Security`); it keeps the keyboard focus, so in run 43692189824
  every SendKeys space of GO went to it and cinq never fired a cue (no warning
  either): `shots/arm-prompt-took-the-keys.png`.  Enter, Tab then Enter, and
  Alt+A (after focusing it) did not answer it; a mouse click on Allow at
  (414, 527) did and turned both rules into Allow (run 43696223664).
  Packets between the computer's own addresses reached the listener while
  the prompt was open, so the firewall does not filter them.
- **WSL window.**  On `windows-11-arm` the "Windows Subsystem for Linux must
  be updated" window opened by itself and was closed 7 to 9 times per job
  (before each GO that found it); never on `windows-latest`.  Without
  closing it, SendKeys goes there and starts a WSL download.
- **Desktop.**  1024x768; cinq opens a 1456x939 window, so the sidebar is
  off screen in captures.
- The runners got different addresses in each run (10.1.0.19, .107, .110,
  .119; Hyper-V 172.x).

## Eos on aarch64 (stretch), runs 43692189824 and 43693917753

| Step | Result |
| --- | --- |
| Install (VC++ x64 runtime 26 s, MSI 15 s) | worked; the x86 runtime returned 1638 (a newer one present) |
| Start, Offline w/viz | `Eos : 1` main window 40 s after the launcher, `Eos.exe` 860 MB, listening on 3036, 3037, 3038 |
| OSC on TCP 3037 with SLIP | answered `/eos/ping` with its state at once; `/eos/newcmd` programmed a patch of channels 1 to 3 and cues 1 to 3 |
| cinq network cue fires | cinq's GOs fired `/eos/out/event/cue/1/1/fire` at 36.99 s, `1/2` at 45.37 s (the `go_0` key), `1/3` at 53.84 s, and `1/2` at 79.05 s; the cue to 127.0.0.1:3999 warned once |
| Eos light | sACN priority 1 from two addresses; channel 1 faded to 179 over about 3 s (cue time), channels 2 and 3 to 204 and 255 over 5 s |

Screenshots: `shots/arm-eos-programmed.png`,
`shots/arm-eos-cinq-network-cue-2.png`.  The firewall was off in this job
(`install.ps1` turns it off), so no prompt came up.

## Suspected cinq bugs

None found.  Notes only: cinq sends zeros on all streams before the first
cue, as the man page does not say; a refused TCP port on Windows always reads
`connection timed out`.

## Artifacts

`cues-x86_64-windows`, `cues-aarch64-windows` (`marks.txt`, `wire.pcap`,
`dmx.jsonl`, `console-*.txt`, `cinq-*-err.txt`, `shots/`), `eos-fired`,
`firewall-windows-11-arm`, `firewall-windows-latest`.
