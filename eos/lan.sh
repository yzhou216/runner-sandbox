#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Runs ion-test.toml, the show for the Ion Xe, against a stand-in console at
# the Ion Xe's address, 10.101.100.20, in a network namespace of its own,
# at the end of a cable (a veth pair) on which the runner is 10.101.50.50,
# as a laptop beside the console is, while the runner's own network keeps
# the default route, as Wi-Fi left on does.
#
# The cable starts out, as when cinq starts before it goes in.  Once in,
# ion-test.toml runs through: three network cues and two light cues on
# universe 999.  Then cable.toml sends a cue, the cable comes out for 15
# seconds, a cue goes while it is out, and two more once it is back.  Then
# the cable comes out again, and a cue goes a second later, before it goes
# back in seven seconds after: a cue that TCP held while the cable was out
# and delivered after it went back in would fire late (13 seconds late in
# run 43622465685, before cinq made a new connection after each quiet
# second).
#
# Usage: lan.sh FONTS OUT, with cinq, sway, wtype, grim, and python3 on
# PATH, sudo, and FONTS the directory of the DejaVu fonts
set -euo pipefail

here=$(dirname "$(readlink -f "$0")")
fonts=$(readlink -f "$1")
out=$(mkdir -p "$2" && readlink -f "$2")
work=$(mktemp -d)
python=$(command -v python3)

export HOME=$work/home XDG_RUNTIME_DIR=$work/run FONTCONFIG_FILE=$work/fonts.conf
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
printf '<?xml version="1.0"?><fontconfig><dir>%s</dir><cachedir>%s/fc</cachedir></fontconfig>\n' "$fonts" "$work" > "$FONTCONFIG_FILE"
printf 'output HEADLESS-1 resolution 1600x1000\n' > "$work/sway.config"
printf 'pcm.!default { type null }\nctl.!default { type null }\n' > "$HOME/.asoundrc"

mark() { printf '%s %s\n' "$(date +%s.%N)" "$1" >> "$out/marks.txt"; }
console() { sudo ip netns exec console "$@"; }
cable() { sudo ip link set light0 "$1"; mark "cable $1"; }

sudo ip netns add console
trap 'sudo ip netns del console 2> /dev/null || true; sudo ip link del light0 2> /dev/null || true' EXIT
sudo ip link add light0 type veth peer name console0 netns console
# Without a broadcast address, as ip address add leaves it, which cinq has
# to work out for Art-Net
sudo ip addr add 10.101.50.50/16 dev light0
console ip addr add 10.101.100.20/16 dev console0
console ip link set lo up
console ip link set console0 up
# Groups of sACN reach the console's only network
console ip route add 224.0.0.0/4 dev console0
ip -4 addr > "$out/ip.txt"
ip -4 route >> "$out/ip.txt"

console "$python" "$here/listen.py" "$out/dmx.jsonl" 200 "$out/wire.pcap" &
listener=$!
console "$python" "$here/fakeeos.py" "$out/console.txt" 200 3037 8000 10.101.100.20 &
sway -c "$work/sway.config" > "$out/sway.log" 2>&1 &
for _ in $(seq 100); do
  socket=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'wayland-[0-9]' -printf '%f\n' | head -1)
  [ -n "$socket" ] && break
  sleep 0.1
done
export WAYLAND_DISPLAY=$socket
SWAYSOCK=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'sway-ipc.*.sock' | head -1)
export SWAYSOCK
until swaymsg -t get_version > /dev/null 2>&1; do sleep 0.5; done

go() {
  swaymsg -q '[app_id="cinq"] focus'
  wtype -s 300 ' '
  mark "GO $1"
  sleep "${2:-3}"
}

cinq "$here/../ion-xe/ion-test.toml" > "$out/ion-test.log" 2>&1 &
show=$!
sleep 6
grim "$out/1-cable-out.png" || true
cable up
sleep 3
go ping
go fire
go key
go 'universe 999 up' 4
grim "$out/2-ion-test.png" || true
go 'universe 999 out' 4
kill "$show"
wait "$show" || true

cinq "$here/cable.toml" > "$out/cable.log" 2>&1 &
show=$!
sleep 6
go before
cable down
sleep 15
console ss -tn > "$out/sockets.txt" 2>&1 || true
grim "$out/3-cable-pulled.png" || true
go 'while out'
grim "$out/4-go-while-out.png" || true
cable up
sleep 3
go after
go 'after again'
grim "$out/5-cable-back.png" || true
# Out for a moment, just before a GO
cable down
sleep 1
go blip 7
cable up
sleep 30
grim "$out/6-blip.png" || true
go 'after blip'
kill "$show"
wait "$show" || true
swaymsg exit > /dev/null 2>&1 || true
wait "$listener" || true

# What arrived, and how long after its GO, against what each GO should
# have sent: everything but what went while the cable was out
"$python" - "$out" <<'CHECK' | tee "$out/verdict.txt"
import re, sys
out = sys.argv[1]
gos = [line.split(" ", 2)[0::2] for line in open(f"{out}/marks.txt").read().splitlines() if " GO " in line]
gos = [(float(t), name.removeprefix("GO ")) for t, name in gos]
arrived = {}
for line in open(f"{out}/console.txt"):
    match = re.match(r"(\S+) tcp (/\S+) (.*)", line)
    if match:
        args = re.findall(r"'([^']*)'", match[3])
        name = {"/eos/ping": "ping", "/eos/cue/1/1/fire": "fire", "/eos/key/go_0": "key"}.get(match[2], " ".join(args))
        arrived.setdefault(name, float(match[1]))
late = {"while out", "blip"}
wrong = False
for when, name in gos:
    if name.startswith("universe"):
        continue
    delay = arrived.get(name)
    shown = "never" if delay is None else f"{(delay - when) * 1000:.0f} ms after GO"
    good = (delay is None) if name in late else (delay is not None and delay - when < 1)
    wrong |= not good
    print(f"{'ok' if good else 'WRONG'}: {name}: {shown}")
# Universe 999 of the light cues, at full and half, then out, from the
# runner's end of the cable
import json
heard = [json.loads(line) for line in open(f"{out}/dmx.jsonl")]
for proto, universe in (("sacn", 999), ("artnet", 998)):
    levels = [h["levels"] for h in heard if h.get("proto") == proto and h.get("universe") == universe and h.get("from") == "10.101.50.50"]
    names = {(h.get("name"), h.get("priority")) for h in heard if h.get("proto") == proto and h.get("universe") == universe}
    good = {"1": 255, "2": 128} in levels and levels[-1:] == [{}]
    wrong |= not good
    print(f"{'ok' if good else 'WRONG'}: {proto} universe {universe}: {len(levels)} changes, last {levels[-1:]}, sources {sorted(names, key=str)}")
sys.exit(wrong)
CHECK
