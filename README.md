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
in `cinq/`.

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
  `macos-latest`, first as a shell starts its program and then as Finder
  opens it, and presses Allow on Darwin's Local Network prompt with `ui`,
  built from `screen.swift`, which reads the screen with Vision.

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
| 43547670732 | Allow pressed; cinq.app opened | See below |

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
  answers.
