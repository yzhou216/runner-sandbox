#!/bin/bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# On the arm64 runner itself: upstream's xnulinux-boot.sh builds fki.ko and
# xnu.ko against the runner's Ubuntu kernel and boots PureDarwin on the free
# kernel collection.  Without a carve-out on the kernel command line, XNU's
# memory comes from alloc_contig_pages.  Then commands on the console, an
# xterm on it under Xvfb, and XNU's framebuffer, all saved to OUT.
#   boot-native.sh XNULINUX KC DISKDIR OUT
set -euo pipefail
X=$1 KC=$2 DD=$3 OUT=$4
S=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$OUT"
sudo mkdir -p /lib/firmware/xnu
sudo cp "$KC" /lib/firmware/xnu/kc-free.bin
cp --sparse=always "$DD/pd.img" /var/tmp/pd.img

console() { sudo dmesg | sed -n 's/^.*fki: darwin console: //p'; }
send() { printf '%s\n' "$1" >> /var/tmp/console-in; }
save() {
  console > "$OUT/$1-console.txt" || true
  sudo dmesg > "$OUT/$1-dmesg.txt" || true
  cp /var/tmp/xnu-insmod.log "$OUT/$1-insmod.log" 2>/dev/null || true
  [ -s /var/tmp/xnu-fb.raw ] && python3 -I "$X/tools/hosted/fb2png.py" /var/tmp/xnu-fb.raw 1280 800 "$OUT/$1-fb.png" || true
}
check_run() {
  if sudo dmesg | grep -aq "XNU panic"; then save panic; echo "the hosted kernel panicked" >&2; exit 1; fi
  if ! pgrep -f "insmod xnu-ko/xnu.ko" >/dev/null; then save ended; echo "insmod returned: the run ended" >&2; exit 1; fi
}
wait_console() {	# wait_console TEXT TIMEOUT [RETRY]
  local deadline=$((SECONDS + $2))
  until console | grep -qF -- "$1"; do
    [ $SECONDS -lt $deadline ] || { save timeout; echo "no '$1' on the console after $2 s" >&2; exit 1; }
    check_run
    [ -z "${3:-}" ] || send "$3"
    sleep 15
  done
}
run() { send "$2; echo xnulinux-done-$1-\$((6*7))"; wait_console "xnulinux-done-$1-42" 600; }

cd "$X"
# fki.ko without a carve-out: xnulinux-boot.sh keeps a loaded fki.ko built
# from the same source, and otherwise loads it with the carve-out of its 16 GB
# VM, which is the runner's RAM
make -s -C "/lib/modules/$(uname -r)/build" M="$PWD/fki-ko" modules
sudo insmod fki-ko/fki.ko carve_mb=0
start=$SECONDS
MAC_CONF=$DD/pd.conf DISK=/var/tmp/pd.img IMAGE=xnu/kc-free.bin TRUSTCACHE='' GPU=off RAM_MB=${RAM_MB:-2048} \
  BOOTARGS="amfi_get_out_of_my_way=1 lp-sip0=2050 bt-transport-override=10 -v" bash scripts/xnulinux-boot.sh ||
  { save boot-failed; exit 1; }
deadline=$((SECONDS + 1800))
until sudo dmesg | grep -aq "fki: darwin console:"; do
  [ $SECONDS -lt $deadline ] || { save timeout; echo "no console output" >&2; exit 1; }
  check_run; sleep 5
done
echo "first console line after $((SECONDS - start)) s"
wait_console "xnulinux-42" 1800 'echo xnulinux-$((6*7))'
echo "shell after $((SECONDS - start)) s"
save boot
run 1 "uname -a"
run 2 "sysctl kern.ostype kern.osrelease kern.version hw.ncpu hw.memsize hw.pagesize"
run 3 "sw_vers"
run 4 "ls /"
run 5 "launchctl list"
run 6 "mount"
save shell

# An xterm on the console, typed into with xdotool, as screenshots
noecho=0; console | grep -qF 'echo xnulinux-$((6*7))' && noecho=1
Xvfb :99 -screen 0 1280x800x24 & sleep 3
export DISPLAY=:99
xterm -geometry 160x50+0+0 -fa 'DejaVu Sans Mono' -fs 10 -e bash -c "uname -a; lsmod | grep -E '^(fki|xnu) '; XNU_CONSOLE_NOECHO=$noecho bash $S/console.sh --new" &
sleep 5
for c in "uname -a" "sysctl kern.version hw.ncpu hw.memsize" "ls /" "launchctl list"; do
  xdotool type --delay 50 "$c"; xdotool key Return; sleep 10
done
ffmpeg -loglevel error -f x11grab -video_size 1280x800 -i :99 -frames:v 1 "$OUT/xterm.png"
save screens

send "sync; halt"
for _ in $(seq 1 180); do pgrep -f "insmod xnu-ko/xnu.ko" >/dev/null || break; sleep 5; done
sudo dmesg | grep -a "the hosted system halted" && echo "halted from inside"
if [ -d /sys/module/xnu ]; then sudo rmmod xnu; fi
save halt
