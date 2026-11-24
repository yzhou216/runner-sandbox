# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Helpers of the idle checks, sourced after OUT and APPS are set.  Builds both cinq
# apps' shows and measures, on top of common.sh.

# shellcheck source=common.sh
. darwin/common.sh

W=$PWD/work
mkdir -p "$W/empty" "$OUT"
sw_vers > "$OUT/system.txt"

# tone FILE SECONDS: a 22050 Hz mono tone of that length
tone() {
  python3 - "$1" "$2" << 'PY'
import math, struct, sys, wave
w = wave.open(sys.argv[1], "wb")
w.setnchannels(1); w.setsampwidth(2); w.setframerate(22050)
w.writeframes(b"".join(struct.pack("<h", int(8000 * math.sin(2 * math.pi * 440 * i / 22050))) for i in range(22050 * int(sys.argv[2]))))
w.close()
PY
}

# templaten DIR COUNT: a show of COUNT audio cues, each with a 2 s file of its own, and a 5 s file long.wav
templaten() {
  mkdir -p "$1/sounds"
  tone "$1/sounds/s001.wav" 2
  tone "$1/sounds/long.wav" 5
  python3 - "$1" "$2" << 'PY'
import shutil, sys
d = sys.argv[1]
n = int(sys.argv[2])
for i in range(2, n + 1):
    shutil.copyfile(f"{d}/sounds/s001.wav", f"{d}/sounds/s{i:03d}.wav")
with open(f"{d}/show.cinq.toml", "w") as f:
    f.write('[[list]]\nname = "Main"\n')
    for i in range(1, n + 1):
        f.write(f'\n[[list.cue]]\nid = {i}\nnumber = "{i}"\nkind = "audio"\nname = "Cue {i}"\nfile = "sounds/s{i:03d}.wav"\nlevel = 0.0\n')
PY
}

# templatelights DIR: the lights demo with a sound for its Thunder cue
templatelights() {
  mkdir -p "$1/sounds"
  tone "$1/sounds/thunder.wav" 3
  cp darwin/lights.cinq.toml "$1/show.cinq.toml"
}

# fresh NAME TEMPLATE: a copy of the template for one launch, and the path of its show
fresh() {
  cp -R "$W/$2" "$W/run-$1"
  echo "$W/run-$1/show.cinq.toml"
}
# conf NAME TEXT: a configuration directory holding TEXT as config.toml
conf() { mkdir -p "$W/conf-$1"; printf '%s\n' "$2" > "$W/conf-$1/config.toml"; echo "$W/conf-$1"; }
GO='[keys]
go = ["Space", "F9"]'

# secs TIME: ps's [[h:]m:]s.cc as seconds
secs() { awk -F: '{ s = 0; for (i = 1; i <= NF; i++) s = s * 60 + $i; printf "%.2f", s }' <<< "$1"; }
cpu() { secs "$(ps -o time= -p "$PID" | tr -d ' ')"; }
# idle NAME: percent of one core that cinq used in 60 s, 20 s after the launch
idle() {
  sleep 20
  focus
  ps -o pid,time,%cpu,rss -p "$PID" > "$D/ps-before.txt" 2>&1 || true
  a=$(cpu)
  t0=$(now)
  sleep 60
  b=$(cpu)
  t1=$(now)
  awk -v a="$a" -v b="$b" -v t0="$t0" -v t1="$t1" -v n="$1" \
    'BEGIN { printf "%s: %.2f s of processor in %.2f s = %.2f%% of one core\n", n, b - a, t1 - t0, 100 * (b - a) / (t1 - t0) }' | tee -a "$OUT/idle.txt"
  ps -M -p "$PID" > "$D/threads.txt" 2>&1 || true
}
# answer NAME: presses Allow on each Local Network prompt that comes within 25 s
answer() {
  n=0
  while [ "$n" -lt 4 ] && ./ui click Allow 25 > "$D/allow-$n.txt" 2>&1; do
    n=$((n + 1))
    mark "pressed Allow ($1, press $n)"
    sleep 2
  done
  echo "$1: $n Allow presses" | tee -a "$OUT/prompts.txt"
}
# burst DIR: screenshots as fast as they go, each stamped before and after, until DIR/stop exists
burst() {
  mkdir -p "$1"
  for i in $(seq -w 1 999); do
    [ -e "$1/stop" ] && break
    before=$(now)
    screencapture -x -t jpg "$1/f$i.jpg"
    printf '%s %s %s\n' "$i" "$before" "$(now)" >> "$1/stamps.txt"
  done
}
# readframes DIR: the text of every frame of a burst, in f*.txt
readframes() {
  for f in "$1"/f*.jpg; do
    ./ui ocr "$f" > "${f%.jpg}.txt" 2>&1 || true
  done
}
# waitgone PID SECONDS: seconds until the process ends, or "alive" after SECONDS
waitgone() {
  start=$(now)
  while kill -0 "$1" 2> /dev/null; do
    elapsed=$(awk -v a="$start" -v b="$(now)" 'BEGIN { print b - a }')
    if awk -v e="$elapsed" -v m="$2" 'BEGIN { exit !(e > m) }'; then echo alive; return; fi
    sleep 0.05
  done
  awk -v a="$start" -v b="$(now)" 'BEGIN { printf "%.2f\n", b - a }'
}
template500() { templaten "$1" 500; }
