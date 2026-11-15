#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Runs inside a container of a distribution: installs cinq's package with the
# distribution's own tool, checks what it put on the system, and, where cinq
# starts, runs show.toml in headless sway beside listen.py and fakeeos.py.
#
# Usage: run.sh KIND, KIND being deb, rpm, or arch, with the package in /pkg,
# this directory at /w, and /out for results
set -uo pipefail

kind=$1
out=/out
here=$(dirname "$(readlink -f "$0")")
mkdir -p "$out"

say() { printf '%s\n' "$*" | tee -a "$out/report.txt"; }
mark() { printf '%s %s\n' "$(date +%s.%N)" "$1" >> "$out/marks.txt"; }
# wtype sleeps 300 ms before it types, so the key lands that long after this
pressed() { printf '%s %s\n' "$(awk -v t="$(date +%s.%N)" 'BEGIN { printf "%.3f", t + 0.3 }')" "$1" >> "$out/marks.txt"; }

{
  # shellcheck source=/dev/null
  . /etc/os-release
  echo "$PRETTY_NAME"
  uname -m
  ldd --version 2>&1 | head -1
} > "$out/system.txt"
cat "$out/system.txt" | tee -a "$out/report.txt"
ip -4 addr > "$out/ip.txt" 2>&1 || true

# 1. The package, on the bare image, through the distribution's own tool
case $kind in
  deb)
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq > "$out/update.txt" 2>&1
    pkg=$(echo /pkg/*.deb)
    apt-get install -y "$pkg" > "$out/install.txt" 2>&1
    status=$?
    [ $status -eq 0 ] || dpkg -i --force-depends "$pkg" > "$out/install-forced.txt" 2>&1
    list() { dpkg -L cinq; }
    ;;
  rpm)
    pkg=$(echo /pkg/*.rpm)
    if command -v dnf > /dev/null; then
      dnf install -y "$pkg" > "$out/install.txt" 2>&1
    else
      zypper --non-interactive --no-gpg-checks install --allow-unsigned-rpm "$pkg" > "$out/install.txt" 2>&1
    fi
    status=$?
    [ $status -eq 0 ] || rpm -i --nodeps "$pkg" > "$out/install-forced.txt" 2>&1
    list() { rpm -ql cinq; }
    ;;
  arch)
    pkg=$(echo /pkg/*.pkg.tar.zst)
    pacman -Syu --noconfirm > "$out/update.txt" 2>&1
    pacman -U --noconfirm "$pkg" > "$out/install.txt" 2>&1
    status=$?
    [ $status -eq 0 ] || pacman -Udd --noconfirm "$pkg" > "$out/install-forced.txt" 2>&1
    list() { pacman -Ql cinq; }
    ;;
esac
say "install with the distribution's tool: exit $status"
tail -n 15 "$out/install.txt" | sed 's/^/  | /' | tee -a "$out/report.txt"
list > "$out/files.txt" 2>&1

binary=$(command -v cinq || echo /usr/bin/cinq)
say "binary: $binary"
cinq --version > "$out/version.txt" 2>&1
say "cinq --version: exit $? $(head -c 300 "$out/version.txt")"
ldd "$binary" > "$out/ldd.txt" 2>&1
say "ldd lines that say not found or an error: $(command grep -c -E 'not found|version .* not|error' "$out/ldd.txt")"
command grep -E 'not found|version .* not|error' "$out/ldd.txt" | sed 's/^/  | /' | tee -a "$out/report.txt"
say "man pages: $(find /usr/share/man -name 'cinq*' 2> /dev/null | tr '\n' ' ')"
say "desktop file: $(ls /usr/share/applications/cinq.desktop 2>&1)"
say "mime file: $(ls /usr/share/mime/packages/cinq.xml 2>&1)"
command grep -E '^(Exec|Name|MimeType|Categories)=' /usr/share/applications/cinq.desktop 2> /dev/null | sed 's/^/  | /' | tee -a "$out/report.txt"

if ! cinq --version > /dev/null 2>&1; then
  # Libraries of the package's dependencies by hand, to see what stays
  # missing beyond them
  case $kind in
    deb) apt-get install -y libasound2t64 libgcc-s1 libdbus-1-3 libwayland-client0 libxkbcommon0 > "$out/libs.txt" 2>&1 ||
      apt-get install -y libasound2 libgcc-s1 libdbus-1-3 libwayland-client0 libxkbcommon0 >> "$out/libs.txt" 2>&1 ;;
    rpm) if command -v dnf > /dev/null; then
        dnf install -y alsa-lib libgcc dbus-libs libwayland-client libxkbcommon > "$out/libs.txt" 2>&1
      else
        zypper --non-interactive install alsa-lib libgcc_s1 libdbus-1-3 libwayland-client0 libxkbcommon0 > "$out/libs.txt" 2>&1 ||
          zypper --non-interactive install libasound2 libgcc_s1 libdbus-1-3 libwayland-client0 libxkbcommon0 >> "$out/libs.txt" 2>&1
      fi ;;
    arch) pacman -S --noconfirm alsa-lib gcc-libs dbus wayland libxkbcommon > "$out/libs.txt" 2>&1 ;;
  esac
  cinq --version > "$out/version-after-libs.txt" 2>&1
  say "after installing its libraries by hand, cinq --version: exit $? $(head -c 300 "$out/version-after-libs.txt")"
  ldd "$binary" 2>&1 | command grep -E 'not found|version .* not' | sed 's/^/  | /' | tee -a "$out/report.txt"
fi
if ! cinq --version > /dev/null 2>&1; then
  say "cinq does not start here, so the light and network tests stay out"
  exit 0
fi

# 2. Tools of the test, from the distribution
case $kind in
  deb)
    apt-get install -y sway wtype grim python3 fonts-dejavu-core iproute2 man-db > "$out/tools.txt" 2>&1 ;;
  rpm)
    if command -v dnf > /dev/null; then
      dnf install -y sway wtype grim python3 dejavu-sans-mono-fonts iproute man-db > "$out/tools.txt" 2>&1
    else
      zypper --non-interactive install sway wtype grim python3 dejavu-fonts iproute2 man > "$out/tools.txt" 2>&1
    fi ;;
  arch)
    pacman -S --noconfirm sway wtype grim python ttf-dejavu iproute2 man-db > "$out/tools.txt" 2>&1 ;;
esac
say "tools install: exit $?"
for f in /usr/share/man/man1/cinq.1* /usr/share/man/man5/cinq-show.5*; do
  [ -e "$f" ] && say "man page $f renders $(man -l "$f" 2> /dev/null | wc -l) lines"
done

work=$(mktemp -d)
export HOME=$work/home XDG_RUNTIME_DIR=$work/run
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1 LIBSEAT_BACKEND=noop
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
printf 'output HEADLESS-1 resolution 1600x1000\n' > "$work/sway.config"
printf 'pcm.!default { type null }\nctl.!default { type null }\n' > "$HOME/.asoundrc"

python3 "$here/listen.py" "$out/dmx.jsonl" 90 "$out/wire.pcap" &
listener=$!
python3 "$here/fakeeos.py" "$out/console-1.txt" 90 &
console=$!
sway -c "$work/sway.config" > "$out/sway.log" 2>&1 &
swaypid=$!
for _ in $(seq 100); do
  socket=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'wayland-[0-9]' -printf '%f\n' | head -1)
  [ -n "$socket" ] && break
  sleep 0.1
done
export WAYLAND_DISPLAY=$socket
SWAYSOCK=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'sway-ipc.*.sock' | head -1)
export SWAYSOCK
for _ in $(seq 60); do
  swaymsg -t get_version > /dev/null 2>&1 && break
  sleep 0.5
done
say "sway: $(swaymsg -t get_version 2>&1 | head -c 120)"

# Standard error of cinq, each line behind its time
cinq "$here/show.toml" 2>&1 | while IFS= read -r line; do
  printf '%s %s\n' "$(date +%s.%N)" "$line"
done > "$out/cinq.err" &
sleep 6
swaymsg -t get_tree | command grep -c '"app_id": "cinq"' > "$out/window.txt"
say "sway alive after 6 s: $(kill -0 "$swaypid" 2>&1 && echo yes)"
say "cinq windows in sway: $(cat "$out/window.txt")"
grim "$out/start.png" 2> /dev/null || true

go() {
  swaymsg -q '[app_id="cinq"] focus'
  pressed "GO $1"
  wtype -s 300 ' '
}
go 1; sleep 3
go 2; sleep 6
go 3; sleep 3
go 4; sleep 3
go 5; sleep 2.5
grim "$out/refused.png" 2> /dev/null || true
sleep 1.5
go 6; sleep 3
kill "$console"
mark "console down"
sleep 1
go 7; sleep 3
python3 "$here/fakeeos.py" "$out/console-2.txt" 40 &
mark "console up"
sleep 1
go 8; sleep 3
go 9; sleep 14
grim "$out/end.png" 2> /dev/null || true
mark end
wait "$listener"
swaymsg exit > /dev/null 2>&1 || true
sleep 1
python3 "$here/analyze.py" "$out" > "$out/summary.txt" 2>&1
cat "$out/summary.txt"
