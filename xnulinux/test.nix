# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# NixOS VM with linuxPackages.xnulinux from the patched Nixpkgs, which loads
# fki.ko and xnu.ko and boots PureDarwin's userland on the free kernel
# collection.  On a GitHub-hosted arm64 runner, which has no KVM, QEMU runs it
# under TCG with a CPU that has PAC.
{
  nixpkgs,
  kc,
  disk,
  rootUuid,
}:
let
  pkgs = import nixpkgs { system = "aarch64-linux"; };

  # Lines of XNU's /dev/console, which fki.ko logs
  consoleLines = "sed -un 's/^.*fki: darwin console: //p'";

  # Interactive terminal on XNU's /dev/console, like upstream's xnu-console.sh
  xnuConsole = pkgs.writeShellScriptBin "xnu-console" ''
    if [ "''${1:-}" = --new ]; then follow=-W; else follow=-w; fi
    [ "''${XNU_CONSOLE_NOECHO:-0}" = 1 ] && stty -echo
    dmesg "$follow" | ${consoleLines} &
    trap 'kill $!' EXIT
    while IFS= read -r line; do printf '%s\n' "$line" >> /var/tmp/console-in; done
  '';
in
pkgs.testers.runNixOSTest {
  name = "xnulinux-puredarwin";
  globalTimeout = 3 * 3600;
  meta.timeout = 3 * 3600;

  nodes.machine =
    { config, pkgs, ... }:
    {
      boot.extraModulePackages = [ config.boot.kernelPackages.xnulinux ];
      # Linux keeps the first 4 GiB of the VM's 8, and fki.ko's carve-out is
      # the memory above it
      boot.kernelParams = [
        "mem=4G"
        "nokaslr"
        "log_buf_len=128M"
      ];
      hardware.firmware = [
        (pkgs.runCommand "xnulinux-kc-free" { } ''
          install -Dm644 ${kc} $out/lib/firmware/xnu/kc-free.bin
        '')
      ];
      services.getty.autologinUser = "root";
      environment.systemPackages = [ xnuConsole ];
      virtualisation = {
        memorySize = 8192;
        cores = 4;
        diskSize = 4096;
        # Last -cpu wins; the implementation-defined PAC algorithm is much
        # faster under TCG than QARMA5
        qemu.options = [ "-cpu max,pauth-impdef=on" ];
      };
    };

  testScript = ''
    import datetime as dt
    import shlex
    import time

    uuid = "${rootUuid}"
    log_filter = "grep -a 'fki:\\|xnu:' | grep -av 'darwin console:' | tail -n 200"

    def console():
        return machine.succeed("dmesg | ${consoleLines}")

    def check_run():
        if machine.execute("dmesg | grep -aq 'XNU panic'")[0] == 0:
            print(machine.succeed("dmesg | " + log_filter))
            raise Exception("the hosted kernel panicked")
        if machine.execute("systemctl is-active -q xnulinux")[0] != 0:
            print(machine.succeed("journalctl -u xnulinux --no-pager; dmesg | " + log_filter))
            raise Exception("modprobe xnu returned: the run ended")

    def send(line):
        machine.succeed("printf '%s\\n' " + shlex.quote(line) + " >> /var/tmp/console-in")

    def wait_console(text, timeout, retry=None):
        deadline = time.time() + timeout
        while text not in console():
            if time.time() > deadline:
                print(machine.succeed("dmesg | " + log_filter))
                raise Exception(f"no {text!r} on the console after {timeout} s")
            check_run()
            if retry:
                send(retry)
            time.sleep(15)

    def run(i, cmd, timeout=600):
        send(f"{cmd}; echo xnulinux-done-{i}-$((6*7))")
        wait_console(f"xnulinux-done-{i}-42", timeout)

    def save(name):
        machine.succeed(
            f"dmesg | ${consoleLines} > /var/tmp/{name}-console.txt;"
            f" dmesg > /var/tmp/{name}-dmesg.txt; cp /var/tmp/xnu-fb.raw /var/tmp/{name}-fb.raw"
        )
        for f in ["console.txt", "dmesg.txt", "fb.raw"]:
            machine.copy_from_vm(f"/var/tmp/{name}-{f}")

    machine.wait_for_unit("multi-user.target", timeout=dt.timedelta(hours=1))
    machine.succeed("dmesg -n 1")

    with subtest("fki.ko loads with the carve-out"):
        machine.succeed("modprobe fki carve_base=0x140000000 carve_mb=3584")
        machine.succeed("test -c /dev/xnu-hostcall")
        print(machine.succeed("dmesg | grep -a 'fki:'; modinfo fki xnu"))

    with subtest("xnu.ko boots the free kernel collection"):
        machine.succeed("cp --sparse=always ${disk} /var/tmp/pd.img && chmod 644 /var/tmp/pd.img")
        machine.succeed(
            ": > /var/tmp/console-in; : > /var/tmp/xnu-input; : > /var/tmp/xnu-fb.raw;"
            " truncate -s 1M /var/tmp/xnu-keystore.bin"
        )
        # Linux's interrupts off CPU 0, which XNU takes
        machine.succeed(
            "echo e > /proc/irq/default_smp_affinity;"
            " for f in /proc/irq/*/smp_affinity; do echo e > $f 2>/dev/null || true; done"
        )
        args = [
            "mode=3",
            "cpus=1",
            "ram_mb=3072",
            "image=xnu/kc-free.bin",
            "disk=/var/tmp/pd.img",
            f"root_uuid={uuid}",
            f"boot_uuid={uuid}",
            f"preboot_uuid={uuid}",
            "console_in=/var/tmp/console-in",
            "input_in=/var/tmp/xnu-input",
            "fb_dump=/var/tmp/xnu-fb.raw",
            "keystore=/var/tmp/xnu-keystore.bin",
            'bootargs="amfi_get_out_of_my_way=1 lp-sip0=2050 bt-transport-override=10 -v"',
        ]
        start = time.time()
        machine.succeed(
            "systemd-run --unit=xnulinux "
            + " ".join(shlex.quote(a) for a in ["modprobe", "xnu", *args])
        )
        deadline = time.time() + 3600
        while machine.execute("dmesg | grep -aq 'fki: darwin console:'")[0] != 0:
            if time.time() > deadline:
                save("timeout")
                raise Exception("no console output")
            check_run()
            time.sleep(15)
        print(f"first console line after {time.time() - start:.0f} s")

    with subtest("PureDarwin's shell answers on the console"):
        try:
            wait_console("xnulinux-42", 3600, retry="echo xnulinux-$((6*7))")
        finally:
            save("boot")
        print(f"shell after {time.time() - start:.0f} s")
        run(1, "uname -a")
        run(2, "sysctl kern.ostype kern.osrelease kern.version hw.ncpu hw.memsize hw.pagesize")
        run(3, "sw_vers")
        run(4, "ls /")
        run(5, "launchctl list")
        run(6, "mount")
        save("shell")

    with subtest("screenshots"):
        echoes = "echo xnulinux-$((6*7))" in console()
        noecho = "XNU_CONSOLE_NOECHO=1 " if echoes else ""
        machine.wait_for_unit("getty@tty1.service")
        machine.send_chars(f"clear; {noecho}xnu-console\n")
        time.sleep(30)
        machine.screenshot("darwin-console")
        machine.send_key("ctrl-c")
        machine.send_chars(
            "clear; uname -a; lsmod | grep -E '^(fki|xnu) ';"
            " modinfo -F filename fki xnu; " + noecho + "xnu-console --new\n"
        )
        time.sleep(10)
        for cmd in ["uname -a", "sysctl kern.version hw.ncpu hw.memsize", "ls /"]:
            machine.send_chars(cmd + "\n")
            time.sleep(30)
        machine.screenshot("linux-and-darwin")
        machine.send_key("ctrl-c")
        save("screens")

    with subtest("halt from inside ends the run"):
        send("sync; halt")
        machine.wait_until_succeeds(
            "dmesg | grep -aq 'the hosted system halted'", timeout=dt.timedelta(minutes=15)
        )
        machine.wait_until_fails("systemctl is-active -q xnulinux", timeout=dt.timedelta(minutes=5))
        machine.succeed("if [ -d /sys/module/xnu ]; then rmmod xnu; fi; rmmod fki")
        save("halt")
  '';
}
