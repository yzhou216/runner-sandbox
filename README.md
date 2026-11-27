<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# xnulinux on a GitHub-hosted arm64 runner

[AFK-surf/xnulinux](https://github.com/AFK-surf/xnulinux) (803cfe0b) runs a
source-built XNU kernel collection as a Linux kernel module.  This session
tested `linuxPackages.xnulinux`, the Nixpkgs package of its two modules
(`fki.ko`, `xnu.ko`) in [nixpkgs/xnulinux.patch](nixpkgs/xnulinux.patch), by
booting PureDarwin's userland with it.  Nothing of Apple's is involved: the
kernel collection is upstream's free one (`kc-free.bin`, XNU and PureDarwin's
kexts built from source), and the disk is PureDarwin's arm64 root on HFS+.

[.github/workflows/xnulinux.yml](.github/workflows/xnulinux.yml) has these
jobs, each cached by the revisions it builds:

- `kc` (`ubuntu-24.04-arm`): macOS-DroidVM's toolchain and xnu-12377 with
  upstream's `vm-xnu-build.sh`, about 10 minutes, then
  [xnulinux/segaddrs.py](xnulinux/segaddrs.py),
  [xnulinux/kexts.sh](xnulinux/kexts.sh) and upstream's `free-kc/build.sh`.
- `pd` (`ubuntu-24.04`): `image-arm64-virt-minimal` of AFK-surf/PureDarwin
  (b3b4b281) with hydra.lab004.dev as a substituter (62 derivations built,
  20 minutes), and its root as a tarball.
- `disk` (`macos-latest`): upstream's `mk-puredarwin-disk.sh`.
- `module`: the package built for Linux 7.3.5.
- `boot-nixos`: a NixOS VM ([xnulinux/test.nix](xnulinux/test.nix)) with the
  package's modules, under QEMU TCG with `-cpu neoverse-n2,pauth-impdef=on`,
  since the runner has no KVM.
- `boot-native`: upstream's `xnulinux-boot.sh` on the runner's own Ubuntu
  kernel ([xnulinux/boot-native.sh](xnulinux/boot-native.sh)).

## Outcome

Run 45913719742 passed every job.

- NixOS VM: `fki.ko` loads with a 3.5 GiB carve-out above `mem=4G`, XNU
  reaches PureDarwin's zsh 39 s after `modprobe xnu`, the shell runs
  `uname -a`, `sysctl`, `sw_vers`, `ls /`, `launchctl list` and `mount`, and
  `halt` inside ends the run (`fki: xnu: the hosted system halted`), after
  which both modules unload.  The test took 547 s.
- Runner: the GitHub arm64 runner is a Neoverse-N2 Hyper-V guest at EL1 with
  PAC, 4K pages, Secure Boot off and no lockdown (probe run 45896800430), so
  the modules load there too.  With `carve_mb=0`, 2 GiB of XNU memory comes
  from `alloc_contig_pages`, the shell answers 19 s after the boot, and
  `halt` ends the run.

The screenshots are in the `boot-nixos` and `boot-native` artifacts of that
run (`linux-and-darwin.png`, `darwin-console.png`, `xterm.png`).

## Found on the way

- Upstream's free collection does not build on a machine without Apple's
  kernelcache.  `vm-xnu-rebuild.sh` regenerates DroidVM's `k27/segaddrs.sh`
  and builds XnuCPU, XnuDisk and XnuDrm only when `out/kc/kc27.bin` exists, so
  the kernel is linked with DroidVM's committed slots, its 0xae4000-byte
  `__TEXT_EXEC` runs past `__KLD`, and `kclink` fails with "segments
  __TEXT_EXEC (0xf94000-0x38d4000) and __PRELINK_INFO (0x3864000-0x386c000)
  overlap" (run 45898903074).  `segaddrs.py` applies `kc27-layout.py`'s rules
  with Apple's slots read back from the committed list, and the kernel is
  linked again (run 45902083067).
- `fki.ko` calls `kernel_neon_begin()` with a buffer, which Linux takes only
  since 6.19, so the package is marked broken on Linux 6.18 (run 45898903074).
- `fki.ko` walks 4-level page tables and writes classic block descriptors.  On
  QEMU's `max` CPU, the NixOS kernel (52-bit VA) uses LPA2 and 5 levels, so
  `xnu.ko` finds "no L1 table for the top window" (run 45905716384);
  `arm64.nolva` avoids that.  It also has S1PIE, with which Linux reads the
  descriptors through permission indirection as read-only, and `macho_copy`
  faults on "kernel write to read-only memory at virtual address
  fffffff007004000" (run 45909253465).  Neoverse-N2 has neither.
- `xnulinux-boot.sh` loads `fki.ko` with its 16 GB VM's carve-out when none is
  loaded, which overlaps the runner's RAM (run 45904169090).
- Under `pipefail`, `dmesg | grep -q` fails when it matches, because `dmesg`
  dies of SIGPIPE; runs 45905380222 and 45909253465 waited on it although
  the shell was up.
