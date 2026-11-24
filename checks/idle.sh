#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Idle cost, file watching, light news, clean exit, and byte order mark checks
# of cinq on GNU/Linux in headless sway, for an old and a new build.
#
# Usage: idle.sh FONTS OUT CINQ_OLD DEMO_OLD CINQ_NEW DEMO_NEW, with sway,
# wtype, grim, and sudo on PATH, CINQ_* the cinq executables and DEMO_* the
# directories of the demo shows of each build.  HINT_AUDIO and HINT_ITEM are
# hint labels of the Audio device list and of an item in it.
set -uo pipefail
fonts=$(readlink -f "$1")
out=$(mkdir -p "$2" && readlink -f "$2")
cinq_old=$(readlink -f "$3")
demo_old=$(readlink -f "$4")
cinq_new=$(readlink -f "$5")
demo_new=$(readlink -f "$6")
work=$(mktemp -d)

export HOME=$work/home XDG_RUNTIME_DIR=$work/run FONTCONFIG_FILE=$work/fonts.conf
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
printf '<?xml version="1.0"?><fontconfig><dir>%s</dir><cachedir>%s/fc</cachedir></fontconfig>\n' "$fonts" "$work" > "$FONTCONFIG_FILE"
cat > "$work/sway.config" <<'CONF'
output HEADLESS-1 resolution 1280x800
for_window [app_id="cinq"] floating enable, resize set 1100 700, move position center
CONF
if grep -q Dummy /proc/asound/cards 2> /dev/null; then
  printf 'pcm.!default { type plug\n slave.pcm "hw:Dummy,0" }\nctl.!default { type hw\n card Dummy }\n' > "$HOME/.asoundrc"
  echo dummy-card > "$out/sound.txt"
else
  printf 'pcm.!default { type null }\nctl.!default { type null }\n' > "$HOME/.asoundrc"
  echo null-device > "$out/sound.txt"
fi

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
{ uname -a; echo "cores $(nproc), CLK_TCK $(getconf CLK_TCK)"; } > "$out/runner.txt"

hz=$(getconf CLK_TCK)
now() { date +%s.%N; }
say() { printf '%s %s\n' "$(date +%T)" "$*" | tee -a "$out/steps.txt"; }
shot() { grim "$out/$1.png" || true; }
focus() { swaymsg -q '[app_id="cinq"] focus'; sleep 0.5; }
key() { focus; wtype -s 300 -d 100 "$@"; sleep 1; }
alive() { [ -r "/proc/$1/status" ] && ! grep -q '^State:.*Z' "/proc/$1/status"; }

# conf DIR LINE...: config.toml with GO also on F9
conf() {
  local dir=$1
  shift
  mkdir -p "$dir"
  { printf '%s\n' "$@"; printf '\n[keys]\ngo = ["Space", "F9"]\n'; } > "$dir/config.toml"
}

# show500 DIR: 500 audio cues, each with a copy of one short WAV of its own
show500() {
  mkdir -p "$1/sounds"
  {
    printf '[[list]]\nname = "Main"\n'
    for i in $(seq 500); do
      cp "$demo_new/sounds/knock.wav" "$(printf '%s/sounds/f%04d.wav' "$1" "$i")"
      printf '\n[[list.cue]]\nid = %s\nnumber = "%s"\nkind = "audio"\nname = "Cue %s"\nfile = "sounds/f%04d.wav"\n' "$i" "$i" "$i" "$i"
    done
  } > "$1/show.cinq.toml"
}

# start PHASE CINQ CONFIGDIR SHOW [NAMESPACE]: sets cinq to the process ID
start() {
  phase=$1
  : > "$out/$phase.err"
  local run=("$2" --config-dir "$3" "$4")
  if [ -n "${5:-}" ]; then
    sudo env "HOME=$HOME" "XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR" "WAYLAND_DISPLAY=$WAYLAND_DISPLAY" "FONTCONFIG_FILE=$FONTCONFIG_FILE" "ICED_BACKEND=$ICED_BACKEND" ip netns exec "$5" /usr/bin/setpriv --reuid="$(id -u)" --regid="$(id -g)" --clear-groups "${run[@]}" \
      2> >(while IFS= read -r l; do printf '%s %s\n' "$(now)" "$l" >> "$out/$phase.err"; done) &
    sleep 2
    cinq=$(pgrep -n -x cinq)
  else
    "${run[@]}" 2> >(while IFS= read -r l; do printf '%s %s\n' "$(now)" "$l" >> "$out/$phase.err"; done) &
    cinq=$!
  fi
}

# closewin PHASE: closes the window as the user would and times the exit
closewin() {
  local t0 t1
  t0=$(now)
  swaymsg -q '[app_id="cinq"] kill' || true
  while alive "$cinq" && [ "$(awk -v a="$t0" -v b="$(now)" 'BEGIN{print (b-a>10)}')" = 0 ]; do sleep 0.02; done
  t1=$(now)
  if alive "$cinq"; then
    say "$1: process $cinq still alive 10 s after the window closed"
    kill -9 "$cinq" 2> /dev/null || true
  else
    say "$1: process ended $(awk -v a="$t0" -v b="$t1" 'BEGIN{printf "%.2f", b-a}') s after the window closed"
  fi
  sleep 1
}

# ticks PID: processor time of all threads of a process so far, in clock ticks
ticks() { awk '{print $14 + $15}' "/proc/$1/stat"; }

# threads PID: processor ticks of each thread, busiest first
threads() {
  for t in "/proc/$1"/task/*; do
    printf '%s %s\n' "$(awk '{print $14 + $15}' "$t/stat")" "$(cat "$t/comm")"
  done | sort -rn | head -6 | tr '\n' ';'
}

# measure PHASE SECONDS: processor time of cinq over the interval, as percent of one core
measure() {
  local a b
  a=$(ticks "$cinq")
  sleep "$2"
  b=$(ticks "$cinq")
  say "$1: $(awk -v a="$a" -v b="$b" -v hz="$hz" -v s="$2" 'BEGIN{printf "%d ticks in %d s = %.2f percent of one core", b-a, s, (b-a)/hz/s*100}'), $(ls "/proc/$cinq/task" | wc -l) threads, RSS $(awk '/VmRSS/{print $2}' "/proc/$cinq/status") kB; threads by total ticks so far: $(threads "$cinq")"
}

# series NAME SECONDS: screenshots as fast as they come, one kept at each change
series() {
  local end=$((SECONDS + $2)) last='' n=0 t h
  : > "$out/$1.frames"
  while [ "$SECONDS" -lt "$end" ]; do
    t=$(now)
    grim "$work/frame.png" 2> /dev/null || continue
    h=$(md5sum < "$work/frame.png" | cut -d' ' -f1)
    printf '%s %s\n' "$t" "$h" >> "$out/$1.frames"
    if [ "$h" != "$last" ] && [ "$n" -lt 40 ]; then
      n=$((n + 1))
      cp "$work/frame.png" "$out/$1-$n.png"
      printf '%s %s-%s.png\n' "$t" "$1" "$n" >> "$out/$1.changes"
      last=$h
    fi
  done
}
# mark NAME: the moment of an action
mark() { now > "$out/$1.act"; }
# lags NAME: seconds from the action to each change of the screen after it
lags() {
  say "$1: screen changes after the action: $(awk -v a="$(cat "$out/$1.act")" '$1 > a {printf "+%.2f %s ", $1 - a, $2}' "$out/$1.changes")"
}

# 1. Idle cost, old and new
for ver in old new; do
  bin=cinq_$ver
  demo=demo_$ver
  say "== $ver =="
  # 500 cues
  show500 "$work/big-$ver"
  conf "$work/c-big-$ver"
  start "$ver-500" "${!bin}" "$work/c-big-$ver" "$work/big-$ver/show.cinq.toml"
  sleep 20
  shot "$ver-500-0-loaded"
  measure "2 $ver 500 cues idle" 60
  shot "$ver-500-1-after"
  if [ "$ver" = new ]; then
    # 4. File watching, in the same instance
    sounds=$work/big-$ver/sounds
    series 4-delete 9 & sp=$!
    sleep 2
    mv "$sounds/f0001.wav" "$work/f0001.away"
    mark 4-delete
    wait "$sp"
    lags 4-delete
    series 4-restore 9 & sp=$!
    sleep 2
    mv "$work/f0001.away" "$sounds/f0001.wav"
    mark 4-restore
    wait "$sp"
    lags 4-restore
    cp "$demo_new/sounds/thunder.wav" "$sounds/new.wav"
    series 4-replace 9 & sp=$!
    sleep 2
    mv "$sounds/new.wav" "$sounds/f0002.wav"
    mark 4-replace
    wait "$sp"
    lags 4-replace
  fi
  closewin "6 $ver 500 cues"
  say "$ver-500 stderr: $(cat "$out/$ver-500.err")"
  # light patch
  cp -rL "${!demo}" "$work/demo-$ver"
  chmod -R u+w "$work/demo-$ver"
  conf "$work/c-light-$ver"
  start "$ver-light" "${!bin}" "$work/c-light-$ver" "$work/demo-$ver/lights.cinq.toml"
  sleep 20
  shot "$ver-light-0-loaded"
  measure "3 $ver lights demo idle" 60
  if [ "$ver" = new ]; then
    closewin "6 $ver lights demo"
    say "$ver-light stderr: $(cat "$out/$ver-light.err")"
    # held light, an extra measure, in a second instance
    start "$ver-held" "${!bin}" "$work/c-light-$ver" "$work/demo-$ver/lights.cinq.toml"
    sleep 20
  fi
  key -k F9
  shot "$ver-light-1-go"
  sleep 3
  measure "3b $ver lights demo, cue 1 held" 60
  closewin "6b $ver lights demo with cue 1 held"
  say "$ver-light stderr: $(cat "$out/$ver-light.err")"
  [ "$ver" = new ] && say "$ver-held stderr: $(cat "$out/$ver-held.err")"
done

# 5. Light news: no interface in a network namespace, then one comes up
conf "$work/c5"
for how in lo dummy; do
  ns=ns-$how
  sudo ip netns add "$ns"
  sudo ip netns exec "$ns" ip link set lo down
  say "5 $how: interfaces in the namespace: $(sudo ip netns exec "$ns" ip -br addr | tr '\n' ';')"
  start "5-$how" "$cinq_new" "$work/c5" "$work/demo-new/lights.cinq.toml" "$ns"
  say "5 $how: cinq is process $cinq: $(ps -o pid,comm,args -p "$cinq" | tail -1)"
  sleep 8
  shot "5-$how-0-start"
  series "5-$how-go" 8 & sp=$!
  sleep 2
  focus
  wtype -s 300 -k F9
  mark "5-$how-go"
  wait "$sp"
  lags "5-$how-go"
  say "5 $how: stderr so far: $(cat "$out/5-$how.err")"
  series "5-$how-up" 10 & sp=$!
  sleep 2
  if [ "$how" = lo ]; then
    sudo ip netns exec "$ns" ip link set lo up
  else
    sudo ip netns exec "$ns" ip link add d0 type dummy ||
      { sudo ip netns exec "$ns" ip link add d0 type veth peer name d1 && sudo ip netns exec "$ns" ip link set d1 up; }
    sudo ip netns exec "$ns" ip addr add 10.77.0.1/24 broadcast 10.77.0.255 dev d0
    sudo ip netns exec "$ns" ip link set d0 up
  fi
  mark "5-$how-up"
  say "5 $how: interfaces after: $(sudo ip netns exec "$ns" ip -br addr | tr '\n' ';')"
  wait "$sp"
  lags "5-$how-up"
  say "5 $how: stderr: $(cat "$out/5-$how.err")"
  closewin "6 lights demo in namespace $how"
  sudo ip netns del "$ns"
done

# 7. Byte order mark
mkdir -p "$work/s7/sounds" "$work/c7"
cp "$demo_new/sounds/rain.flac" "$work/s7/sounds/"
cat > "$work/s7/show.cinq.toml" <<'TOML'
[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "audio"
name = "Rain"
file = "sounds/rain.flac"
TOML
{ printf '\xEF\xBB\xBF'; printf '[keys]\ngo = ["Space", "F9"]\n'; } > "$work/c7/config.toml"
start 7-bom "$cinq_new" "$work/c7" "$work/s7/show.cinq.toml"
sleep 8
shot 7-bom-0-start
key -k F9
shot 7-bom-1-f9
say "7 first start stderr bytes: $(wc -c < "$out/7-bom.err")"
key f
shot 7-bom-2-hints
if [ -n "${HINT_AUDIO:-}" ]; then
  key "$HINT_AUDIO"
  shot 7-bom-3-list
  if [ -n "${HINT_ITEM:-}" ]; then
    key "$HINT_ITEM"
    shot 7-bom-4-chosen
  fi
fi
closewin "7 first start"
say "7 config.toml after: $(head -c 3 "$work/c7/config.toml" | od -An -tx1 | tr -d ' \n') then: $(tail -c +4 "$work/c7/config.toml" | tr '\n' '|')"
cp "$work/c7/config.toml" "$out/7-config.toml"
start 7-bom2 "$cinq_new" "$work/c7" "$work/s7/show.cinq.toml"
sleep 8
shot 7-bom2-0-start
key -k F9
shot 7-bom2-1-f9
closewin "7 second start"
say "7 second start stderr bytes: $(wc -c < "$out/7-bom2.err")"
say "7 config.toml at the end: $(head -c 3 "$work/c7/config.toml" | od -An -tx1 | tr -d ' \n') then: $(tail -c +4 "$work/c7/config.toml" | tr '\n' '|')"

swaymsg exit > /dev/null 2>&1 || true
