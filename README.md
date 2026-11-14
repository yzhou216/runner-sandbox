<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq beside ETC Eos, offline

The theater of cinq's author runs its lights from an ETC Ion Xe.  This
branch runs Eos, the Ion Xe's software, offline on `windows-latest`, to see
whether cinq's light cues speak the protocols that Eos and the devices
around it speak, and whether cinq can fire the cues of Eos with network
cues, as QLab does.  cinq comes from snapshots of commits not yet pushed,
in `cinq/`, which last held a164d31 with a patch, for this branch alone,
that writes each send that fails on standard error.

## What runs

`.github/workflows/eos.yml`, with the scripts in `eos/`:

- **cinq-windows** and **cinq-darwin** cross-build cinq from `cinq/` with
  Nix on `ubuntu-latest`.
- **windows** installs Eos 3.3.10 from ETC's CDN: only the MSI of Eos from
  the NSIS installer, since the whole installer, run with `/S`, brings
  console patches and USB, dongle, and SLP drivers, and the runner that ran
  it lost its connection (run 43416352917).  `start.ps1` chooses the Eos
  console mode, whose two buttons have only logos, and **Offline w/viz**,
  which sends sACN for visualizers without a license key, by UI
  Automation.  `listen.py` records every sACN and Art-Net packet that
  reaches the runner, as JSON lines and as a pcap for Wireshark.
  `osc.py` programs Eos over OSC, and `beside.ps1` runs cinq beside it.
- **darwin** runs the light cues of `show.toml` in cinq.app on
  `macos-latest`, as Finder opens it, and presses Allow on Darwin's Local
  Network prompt with `ui`, built from `screen.swift`, which reads the
  screen with Vision.  Earlier runs also started its program from a shell,
  opened it twice, and signed a copy again.
- **darwin-native** builds cinq with Nix on `macos-latest` and checks the
  signature of cinq.app.
- **linux** runs `linux.toml` in cinq in headless sway on `ubuntu-latest`,
  with `fakeeos.py` standing in for a console, decoding the network cues
  over TCP with SLIP and over UDP; `stress.sh` sends hundreds of cues; and
  `lan.sh` runs `ion-xe/ion-test.toml` against a stand-in console at the
  Ion Xe's address, 10.101.100.20, in a network namespace at the end of a
  veth cable that comes out and goes back in.
- **powercut** cuts a stand-in console off under cinq with iptables, as a
  lost power supply does, and checks the cues after it comes back.

## Results

| Run | What | Outcome |
| --- | --- | --- |
| 43416352917 | Whole installer with `/S` | Runner lost its connection after 45 minutes, no logs |
| 43462159979 | MSI alone | Installed in 12 s; the registry dump overflowed PowerShell's stack |
| 43464357513 | Start | Stuck on the console mode prompt, whose buttons have no names |
| 43466962112 | Start, OSC on TCP 3032 | Eos up in Offline w/viz; 3032 took the connection but never answered |
| 43510031736 | OSC on 3032, 3037, 3038; cinq beside Eos | 3037 with SLIP answered; Eos patched, recorded, and fired cues over OSC; tshark read Eos's and cinq's sACN on universe 1 |
| 43512936545 | cinq's network cues fire Eos | Eos fired cue 1 and then cue 2 from cinq's two GOs, and faded its lights in their times |
| 43544918926 | cinq's sACN on Darwin | Darwin's Local Network prompt held the packets back |
| 43547670732 | Allow pressed; cinq.app opened | The program from a shell sent once allowed; cinq.app showed its own prompt, with its reason |
| 43548703970 | The exact Allow | `ui click --contains Allow` had pressed Don't Allow; cinq.app's multicast still held back |
| 43549278854 | Sends that fail, logged | `No route to host` (os error 65) for sACN multicast |
| 43550373520 | cinq.app opened again once allowed | Still `No route to host` on every send, before and after Allow |
| 43551360995 | Darwin's privacy log | `found bundle id cinq-94c18a24ea67fdd9 by PID`, `io.yiyuzhou.cinq by UUID`; codesign of a read-only copy failed |
| 43551745057 | Copy signed under `io.yiyuzhou.cinq` | No send failed; its sACN reached the network |
| 43552387371 | cinq.app signed by rcodesign at build | Failed only until Allow, then sent |
| 43553147379 | cinq.app as Nix builds it on Darwin | Signed as `io.yiyuzhou.cinq`, valid on disk, satisfies its Designated Requirement |
| 43553927187 | Every job, with GNU/Linux | All passed (darwin-native on a second attempt, after cache.nixos.org broke off a download); on GNU/Linux sACN left eth0 and the network cues reached `fakeeos.py` as written, over TCP and UDP |
| 43584888758 | b137aa7 as pushed: six kinds of network cue | Eos fired a cue by number, GO, a cue by list and number, and a command line with spaces; a destination that refused took 1 s on Windows and held nothing else up |
| 43594509473 | 5501a62: Eos down and back; load; a Mac's LAN | cinq reconnected to Eos on its own once Eos came back; 100 cues from two GOs and 10 after a restart arrived in order; cinq.app reached a console at the Mac's own address |
| 43597187624 | Power cut, first try | The two consoles of the script fought over port 3037 |
| 43598804720 | Power cut, with keepalive (5f71ab1) and without (5501a62) | Without keepalive the first cue after the cut was lost without a word; with it, every cue arrived |
| 43600907511 | Every job on d14ade8 | All passed but powercut, whose build failed on a test that raced, fixed in 51d0edc |
| 43621864513 | Every job on 51d0edc | Cancelled for 7f5632a, which also warns on standard error of a lost connection |
| 43622465685 | 7f5632a: the Ion Xe's show over a cable that comes out | ion-test.toml reached a stand-in console at 10.101.100.20 as written; a cue sent a second after the cable came out arrived 13 s after its GO, without a warning, once the cable was back; no Art-Net on a network without a broadcast address |
| 43624566796 | 238fcfe: a new connection after each quiet second | The cue a second after the cable came out never arrived, and warned; every other cue arrived within 10 ms; Eos took a connection for each GO and ran ion-test.toml's cues; still no Art-Net on the cable, since GNU/Linux neither sends nor takes broadcasts on an address without `brd` |
| 43625278634 | c9a6db4: sACN universe discovery | Eos and cinq sent discovery every 10 s, which `listen.py` logged as unknown, since it took packets under 126 bytes for broken; `stress.sh` could not bind its console while `linux.sh`'s still held port 8000 |
| 43626553187, 43626694911, 43626762105 | d6b9350 | Cancelled for fixes to the scripts and to cinq |
| 43627453872 | 27b7559: every job, after a review of the network output | All passed: Eos ran ion-test.toml's cues; on the cable, a GO while it was out and one a second after it came out never arrived, and warned, while every other cue arrived within 10 ms; Art-Net, sACN, and discovery of universe 999 reached the stand-in console; 360 cues under stress, memory flat at 55 MB |

What Eos does, offline:

- It answers OSC only on TCP port 3037, framed with SLIP (OSC 1.1), which
  its manual calls the port of third-party integration, always enabled.
  Port 3032, which needs OSC RX in Setup, takes connections and ignores
  them offline.  `/eos/newcmd` with `#` for Enter types into its command
  line, and `/eos/cue/1/fire` and `/eos/key/go_0` fire cues.
- It sends sACN on every network of the computer, universe 1 for its
  universe 1, at priority 1 rather than its usual 100, so that a
  visualizer's feed never takes over from a real console, with
  per-address-priority (start code 0xDD) packets for the addresses it
  patches.  70% goes out as 179 and 80% as 204, as from cinq.
- `Go To Cue Out#` is a syntax error over OSC.

What cinq does:

- Its light cues go out as sACN, priority 100, universe 1 for its universe
  1, and as Art-Net, Port-Address 0 for universe 1, as Eos numbers them, on
  every network of the computer and to the computer itself.  tshark
  decodes both with no expert warnings.
- Its network cues, OSC over TCP with SLIP to `127.0.0.1:3037`, made Eos
  fire `cue/1/1` and then `cue/1/2`; Eos faded channel 1 to 179 over 2.98 s
  and channels 2 and 3 to 204 and 255 over 4.98 s, the times of its cues.
- A TCP connection kept open while its cable is out takes a message
  without a word, and TCP delivers it once the cable goes back in: 13 s
  after its GO in run 43622465685.  So cinq now keeps a connection only
  while messages to its destination follow each other within a second, and
  drops any message that cannot go out within a second of its cue.
- It lists the universes that it sends every ten seconds, as sACN's
  universe discovery asks; Eos does too, for its universe 1.
- GNU/Linux neither sends nor takes Art-Net's directed broadcasts on an
  address set without `brd`.

What Darwin does:

- It asks the operator whether a program may reach devices on the local
  network the first time it tries, and until then sACN multicast fails
  with `No route to host`, while Art-Net broadcast and loopback go out.
- It keeps the answer for cinq.app under the identifier of its bundle,
  `io.yiyuzhou.cinq`, and looks a running program up by the identifier of
  its signature, which the linker had made the name of the file that rustc
  wrote, `cinq-94c18a24ea67fdd9`.  So multicast stayed off after Allow,
  and after a relaunch, until the bundle was signed ad hoc under its own
  identifier, which cinq's build now does with rcodesign.

For the console itself, `ion-xe/CHECKLIST.md` lists what to note and test
at an Ion Xe, with `ion-xe/ion-test.toml`, a show that pings the console
first and sends light only to universe 999.

## Traps

- Eos's launcher shows a Windows paging-file dialog behind it at logon,
  and its main window, `Eos : 1`, has almost no names for UI Automation;
  drive it by OSC on 3037, or by its PC keys (`keys.ps1`, `keys-1.txt`).
- A runner that loses its connection leaves no job log, so upload an
  artifact after each step.
- `Get-ItemProperty | Select-Object *` over `HKLM\SOFTWARE\ETC` overflowed
  PowerShell's stack; `reg query /s` does not.
- Darwin asks whether a program may reach devices on the local network the
  first time it tries, for programs started from the job's shell on behalf
  of `hosted-compute-agent`, and holds the packets back until someone
  answers; a Python listener left unanswered stalled 17 minutes past its
  own deadline.
- `ui click --contains Allow` finds Don't Allow first, at the same height;
  click the exact `Allow`.
- Files from the Nix store are read-only, so `codesign` of a copy needs
  `chmod -R u+w` first.
- GNU/Linux lets a socket join only 20 multicast groups by default; past
  that, joins fail with ENOBUFS, so `listen.py` joins only the universes
  that the tests use.
- On Windows, a TCP connection to a closed port on the same computer times
  out rather than being refused.
