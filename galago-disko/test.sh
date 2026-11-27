#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Installs galago with nixos-anywhere into a QEMU virtual machine with
# an empty NVMe disk, then boots it from that disk, answers the
# passphrase prompts of the initrd on the serial console, and checks
# the layout.  Each phase is one argument: build, installer, install,
# boot, check, stock, and collect.  FLAKE names the flake revision to
# test.

set -euo pipefail

here=$(dirname "$(readlink -f "$0")")
flake=${FLAKE:?}
# On the large /nix volume of nothing-but-nix
work=/nix/vm
out=$work/out
key=$work/id_ed25519
# Throwaway passphrase of the test disk
passphrase=galago-disko-test

ssh_opts=(
  -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null
  -o ConnectTimeout=10 -o LogLevel=ERROR
)

vm() {
  ssh -i "$key" -p 2222 "${ssh_opts[@]}" root@127.0.0.1 "$@"
}

qemu_running() {
  [ -e "$work/qemu.pid" ] && kill -0 "$(cat "$work/qemu.pid")" 2>/dev/null
}

wait_for_exit() {
  local deadline=$((SECONDS + $1))
  while qemu_running; do
    if ((SECONDS > deadline)); then
      echo "Virtual machine still running after $1 seconds" >&2
      return 1
    fi
    sleep 2
  done
}

# Prompts that have appeared on the serial console of machine $1
prompts() {
  { grep -aoi 'enter passphrase[^:]*' "$out/serial-$1.log" || true; } 2>/dev/null
}

# Answers each prompt that has appeared since the machine started
answered=0
answer_prompts() {
  local asked
  asked=$(prompts "$1" | wc -l)
  while ((answered < asked)); do
    sleep 1
    printf '%s\n' "$passphrase" | socat -u - UNIX-CONNECT:"$work/serial.sock"
    answered=$((answered + 1))
    echo "Answered passphrase prompt $answered"
  done
}

wait_for_ssh() {
  local name=$1 deadline=$((SECONDS + $2))
  answered=0
  until vm true 2>/dev/null; do
    answer_prompts "$name"
    if ! qemu_running; then
      echo "QEMU exited before SSH came up" >&2
      return 1
    fi
    if ((SECONDS > deadline)); then
      echo "No SSH after $2 seconds" >&2
      return 1
    fi
    sleep 2
  done
  prompts "$name" | tee "$out/prompts-$name.txt"
  echo "SSH up after $SECONDS seconds, after $answered passphrase prompts"
}

start_qemu() {
  local name=$1
  shift
  local ovmf
  ovmf=$(cat "$work/ovmf")/FV
  rm -f "$work/serial.sock" "$work/monitor.sock"
  qemu-system-x86_64 -name "galago-$name" \
    -enable-kvm -machine q35 -cpu host -smp 4 -m 8192 -no-reboot \
    -drive if=pflash,format=raw,readonly=on,file="$ovmf/OVMF_CODE.fd" \
    -drive if=pflash,format=raw,file="$work/OVMF_VARS.fd" \
    -drive file="$work/nvme.qcow2",if=none,id=nvm,format=qcow2,discard=unmap \
    -device nvme,serial=galagovm0001,drive=nvm \
    "$@" \
    -netdev user,id=net0,hostfwd=tcp:127.0.0.1:2222-:22 \
    -device virtio-net-pci,netdev=net0 \
    -vga std -display none \
    -chardev "socket,id=serial,path=$work/serial.sock,server=on,wait=off,logfile=$out/serial-$name.log" \
    -serial chardev:serial \
    -monitor unix:"$work/monitor.sock",server,nowait \
    -pidfile "$work/qemu.pid" -daemonize
}

case $1 in
  build)
    sudo install -d -o "$(id -u)" -g "$(id -g)" "$work"
    # Avoid copy-on-write under the disk image of the virtual machine
    chattr +C "$work" || true
    mkdir -p "$out"
    ssh-keygen -q -t ed25519 -N '' -C galago-disko -f "$key"

    nix build --print-out-paths -o "$work/tools" --inputs-from "$flake" \
      nixpkgs#qemu_kvm nixpkgs#nixos-anywhere nixpkgs#socat |
      sed 's|$|/bin|' >>"$GITHUB_PATH"
    nix build --print-out-paths -o "$work/ovmf-fd" --inputs-from "$flake" \
      nixpkgs#OVMF.fd >"$work/ovmf"

    nix build --impure -L -o "$work/installer" --file "$here/installer.nix" \
      --argstr flake "$flake" --argstr pubKey "$(cat "$key.pub")"
    for attr in toplevel stock diskoScript; do
      nix build --impure -L -o "$work/$attr" --file "$here/galago.nix" \
        --argstr flake "$flake" "$attr"
    done
    nix path-info -Sh "$work/toplevel" | tee "$out/closure.txt"
    df -h /nix
    ;;

  installer)
    qemu-img create -f qcow2 "$work/nvme.qcow2" 100G
    install -m 644 "$(cat "$work/ovmf")/FV/OVMF_VARS.fd" "$work/OVMF_VARS.fd"
    iso=$(echo "$work"/installer/iso/*.iso)
    start_qemu installer \
      -drive file="$iso",media=cdrom,readonly=on,if=none,id=cd0 \
      -device ide-cd,drive=cd0,bootindex=0
    wait_for_ssh installer 900
    vm 'uname -r; grep VARIANT /etc/os-release; lsblk'
    ;;

  install)
    install -d -m 700 "$work/extra/root" "$work/extra/root/.ssh"
    install -m 600 "$key.pub" "$work/extra/root/.ssh/authorized_keys"
    printf '%s' "$passphrase" >"$work/secret.key"
    nixos-anywhere \
      --store-paths "$(readlink -f "$work/diskoScript")" "$(readlink -f "$work/toplevel")" \
      --disk-encryption-keys /tmp/secret.key "$work/secret.key" \
      -i "$key" --ssh-port 2222 \
      --ssh-option StrictHostKeyChecking=no \
      --ssh-option UserKnownHostsFile=/dev/null \
      --extra-files "$work/extra" \
      --target-host root@127.0.0.1
    # The reboot at the end stops QEMU, which runs with -no-reboot
    wait_for_exit 300
    du -h "$work/nvme.qcow2"
    df -h /nix
    ;;

  boot)
    # No installer this time: OVMF has to find systemd-boot on the disk
    start_qemu disk
    wait_for_ssh disk 900
    # One prompt unlocks the filesystem for all of /, /nix, and /home
    if ((answered != 1)); then
      echo "Expected one passphrase prompt, got $answered" >&2
      exit 1
    fi
    ;;

  check)
    vm 'journalctl -b --no-pager' >"$out/journal-disk.txt" 2>&1 || true
    # Copy the script first, since a command in it may read its stdin
    vm 'cat >/root/check.sh' <"$here/check.sh"
    vm bash /root/check.sh "$(readlink -f "$work/toplevel")" </dev/null |
      tee "$out/check.txt"
    # Fail if the checks did not run to the end
    grep -qx '== Done' "$out/check.txt"
    ;;

  stock)
    # Boot the same system once more, with the unlock units of NixOS
    stock=$(readlink -f "$work/stock")
    NIX_SSHOPTS="-i $key -p 2222 ${ssh_opts[*]}" \
      nix copy --no-check-sigs --to ssh://root@127.0.0.1 "$stock"
    vm "nix-env -p /nix/var/nix/profiles/system --set $stock &&
      $stock/bin/switch-to-configuration boot"
    vm systemctl poweroff || true
    wait_for_exit 300
    start_qemu stock
    wait_for_ssh stock 900
    test "$(vm readlink -f /run/current-system)" = "$stock"
    vm 'systemctl is-system-running --wait; systemctl --failed --no-pager' || true
    echo "Stock unlock units asked for the passphrase $answered times" |
      tee "$out/stock.txt"
    ;;

  collect)
    if qemu_running; then
      echo "screendump $out/screen.ppm" | socat - UNIX-CONNECT:"$work/monitor.sock"
      sleep 2
      vm 'journalctl -b --no-pager' >"$out/journal-final.txt" 2>&1 || true
    fi
    tar -C "$work" -cf "$GITHUB_WORKSPACE/galago-disko.tar" out
    ;;

  *)
    echo "Unknown phase: $1" >&2
    exit 1
    ;;
esac
