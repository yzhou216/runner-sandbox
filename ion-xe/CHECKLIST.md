<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# Testing cinq beside an Ion Xe

## Before you go

- **cinq:** the build of `wip/light-cues` (5501a62), in one of these ways:
  - **With Nix:** `nix build github:yzhou216/cinq/wip/light-cues`.
  - **Without Nix:** artifact `cinq-x86_64-linux-packages` of cinq's
    Packages run 43593847625.  It holds the deb, rpm, and Arch packages,
    and `cinq-0.1.4-aarch64-darwin.pkg` for a Mac with Apple silicon.
  - **Windows:** `cinq-x86_64-windows-built-on-x86_64-linux.exe`, an
    artifact of the "Cross builds for Windows" run 43593847653.
- **Unsigned builds:** none of these has a developer signature.
  - **Darwin:** the installer is refused at first; allow it under System
    Settings > Privacy & Security > Open Anyway, then run it again.
  - **Windows:** SmartScreen warns; choose More info > Run anyway.
- **Test show:** `ion-test.toml`, from this branch.  Cue 1 only pings the
  console.  Cues 2 and 3 fire console cues.  Cues 4 and 5 send light levels
  to universe 999.
- **Monitors:** Wireshark, and sACNView, a free sACN monitor.  In
  Wireshark, enable the ACN protocol's heuristic (Analyze > Enabled
  Protocols > ACN), or sACN shows as plain UDP.
- **Laptop:** an Ethernet port or adapter, and permission to set a static
  address on it.

## At the console: write these down first

1. **Software version.**  It shows on the console's Diagnostics tab (Tab
   99).  Port 3037 needs Eos 3.1 or later.
2. **Address.**  Setup > Device > Network.  The Ion Xe's default is
   10.101.100.20 on port 1 (mask 255.255.0.0) and 192.168.100.20 on
   port 2.  Note which port is cabled to what.
3. **Interface Protocols,** on the same page.  Check that Third Party OSC
   (port 3037) is enabled; it is by default.  Some versions turn it off
   while Allow Remotes is off (Setup > Remotes).
4. **OSC settings.**  Setup > System > Show Control > OSC: OSC RX, OSC TCP
   Mode, and OSC UDP RX Port.
5. **Output protocols.**  Patch > Protocols: sACN and Art-Net on or off,
   the start universes, and the sACN priority (100 by default).
6. **How the wireless DMX is fed.**  Is the transmitter fed by a cable from
   a DMX port of the console, or from the network (sACN or Art-Net)?
   Write down the make and model.

## Laptop setup

1. Turn Wi-Fi off, so that only the lighting network is up.
2. Give the Ethernet port a free static address on the console's network,
   for example 10.101.50.50 with mask 255.255.0.0.  Ask which addresses
   are free first: the 10.101.100.x range is for consoles.
3. Check the link with `ping 10.101.100.20`.
4. **Darwin:** answer Allow when cinq asks to find devices on local
   networks.  If you missed it, turn cinq on under System Settings >
   Privacy & Security > Local Network.
5. **Windows:** allow cinq through the firewall if asked.
6. Start cinq from a terminal, so that its warnings also go to the
   terminal: `cinq ion-test.toml`.

## Tests, in order

1. **Show the console's OSC log.**  On the console, open Tab 99 and turn
   on {Incoming OSC}.
2. **Ping.**  GO on cue 1 of `ion-test.toml`.  `/eos/ping` should show in
   Tab 99, and nothing should change on stage.
3. **Fire a console cue.**  Ask the board operator first.  Cue 2 fires
   cue 1 of the console's cue list 1, and cue 3 presses its GO.  Edit the
   message to a cue that is safe to run.
4. **Light output, only on universe 999.**  GO on cue 4.  sACNView should
   show source "cinq", priority 100, universe 999, with 255 at address 1
   and 128 at address 2.  GO on cue 5 takes them out.  Drive a real
   universe only with the lighting designer's consent, since sACN at the
   same priority merges, highest takes precedence, with the console's
   output on ETC gear.

## When something fails

- **cinq's sidebar** lists every failed network cue under Warnings, with
  its reason, until a message to that destination gets through.  The
  terminal shows the same.
  - *Connection refused:* the console is reachable, but nothing listens
    on the port.  Check Third Party OSC, Allow Remotes, and the port
    number.
  - *Timed out:* nothing answered.  Check the address, the subnet mask,
    the cable, and whether Wi-Fi is taking the route.
  - *No route to host* on Darwin: Local Network permission is off.
- **Wireshark,** with the filter `tcp.port == 3037`:
  - SYN, then SYN-ACK: the connection works.  If Tab 99 still shows
    nothing, check OSC RX.
  - SYN, then RST: refused.
  - SYN, then nothing: not reachable.
- **If port 3037 cannot be used:**
  - Set the console's OSC TCP Mode to OSC 1.1 (SLIP), and use port 3032.
  - Or set an OSC UDP RX Port on the console, such as 8000, and use
    `udp://10.101.100.20:8000` as the destination.
- **Save** the Wireshark capture, a photo of Tab 99, cinq's terminal
  output, and the console's settings above, for me to look at.
