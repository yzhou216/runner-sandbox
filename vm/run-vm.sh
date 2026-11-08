#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Install ReactOS unattended from a boot CD, run the test programs of a
# directory once setup has finished, and collect what they left behind.
#
#   run-vm.sh <boot CD> <i386|amd64> <programs> <output directory>
#
# Needs qemu-system-x86_64, xorriso, mtools, socat, python3, and util-linux.
set -euo pipefail

iso=$(realpath "$1")
arch=$2
programs=$(realpath "$3")
out=$(realpath -m "$4")
here=$(dirname "$(realpath "$0")")
minutes=${MINUTES:-45}

mkdir -p "$out/shots" "$out/results"
work=$(mktemp -d)
trap 'kill $(jobs -p) 2>/dev/null || true; rm -rf "$work"' EXIT
cd "$work"

# Boot CD whose text-mode setup starts at once and runs unattended, then
# starts run.cmd from the CD after the last reboot
xorriso -osirrox on -indev "$iso" -extract /freeldr.ini freeldr.ini \
  -extract "/$arch/unattend.inf" unattend.inf 2>/dev/null
chmod u+w freeldr.ini unattend.inf
sed -i 's/^DefaultOS=.*/DefaultOS=Setup/; s/^TimeOut=.*/TimeOut=1/' freeldr.ini
sed -i 's/^UnattendSetupEnabled = no/UnattendSetupEnabled = yes/' unattend.inf
cat >> unattend.inf <<'EOF'

[GuiRunOnce]
"cmd.exe /c D:\rostest\run.cmd"

[Display]
BitsPerPel = 32
XResolution = 1024
YResolution = 768
EOF
sed -i 's/$/\r/' unattend.inf freeldr.ini
mkdir rostest
cp "$programs"/* rostest/
sed 's/$/\r/' "$here/run.cmd" > rostest/RUN.CMD
xorriso -indev "$iso" -outdev test.iso -boot_image any replay \
  -map freeldr.ini /freeldr.ini -map unattend.inf "/$arch/unattend.inf" \
  -map rostest /ROSTEST -commit 2>&1 | grep -iE 'error|fail' || true

qemu-img create -q -f qcow2 disk.qcow2 8G
accel=tcg
[ -w /dev/kvm ] && accel=kvm
echo "Starting QEMU with $accel"
qemu-system-x86_64 -machine pc,accel=$accel -m 2048 -smp 1 \
  -drive file=disk.qcow2,if=ide,index=0,media=disk \
  -drive file=test.iso,if=ide,index=2,media=cdrom \
  -boot order=c,once=d -vga std -display none \
  -monitor unix:monitor.sock,server=on,wait=off \
  -chardev socket,id=com1,path=com1.sock,server=on,wait=off,logfile="$out/com1.log" \
  -serial chardev:com1 \
  -nic user,model=pcnet,restrict=on -rtc base=localtime &
qemu=$!
python3 "$here/kdbg.py" com1.sock &

# Screenshot every three seconds until ReactOS powers off
deadline=$((SECONDS + minutes * 60))
shot=0
while kill -0 $qemu 2>/dev/null; do
  if [ $SECONDS -ge $deadline ]; then
    echo "Timed out after $minutes minutes"
    kill $qemu
    break
  fi
  shot=$((shot + 1))
  printf 'screendump %s/shots/%05d.png -f png\n' "$out" $shot \
    | socat - UNIX-CONNECT:monitor.sock > /dev/null 2>&1 || true
  sleep 3
done
wait $qemu || true

# Results that run.cmd wrote to C:\results on the FAT system partition
qemu-img convert -O raw disk.qcow2 disk.raw
start=$(sfdisk -d disk.raw 2>/dev/null | awk -F'[=,]' '/start=/ {print $2; exit}' | tr -d ' ' || true)
if [ -n "$start" ]; then
  mcopy -n -s -i "disk.raw@@$((start * 512))" ::/results "$out/results/" || true
fi
find "$out/results" -type f | sort
