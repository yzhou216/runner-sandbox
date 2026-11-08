<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: Rust, iced, and cinq on ReactOS

Asked whether cinq can be built for ReactOS, starting with iced.  `cinq/`
is a snapshot of cinq at f48fe08 (0.1.2), and `proj/` holds small test
programs: `hello` (four threads around a `Mutex`), `one` (one thread, and
the C runtime's math), `threads` (one kind of thread per argument),
and `counter` (iced's counter with cinq's features of iced).  `build.sh`
cross-builds each for `i686-pc-windows-gnullvm` and
`x86_64-pc-windows-gnullvm` with the UCRT, as cinq's own Windows build does,
twice: as it is (`*_ST.EXE`) and with `shim/rosshim.c` linked in
(`*_SH.EXE`).

- `check-imports.sh` lists the imports that a ReactOS boot CD cannot
  resolve, resolving API sets through ReactOS's own table.
- `vm/run-vm.sh` installs ReactOS 0.4.16 unattended in QEMU with KVM from a
  boot CD remastered with `xorriso`, whose `[GuiRunOnce]` starts
  `vm/run.cmd` after the last reboot.  `vm/runner.c` runs each program with
  no error dialogs and logs its output and exit status through
  `OutputDebugString` to COM1, which QEMU writes to `com1.log`;
  `vm/kdbg.py` answers the kernel debugger there, so that a failed
  assertion cannot freeze the machine.
- `windows.ps1` runs the same programs on `windows-latest` as a control.

## Outcome

Run 42470325528 (424beb7) on `ubuntu-latest` and `windows-latest`:

- **As built today, nothing loads.**  ReactOS 0.4.16, the x86 nightly, and
  the x64 nightly (0.4.17-dev-1340-gfaa2d03) each lack 7 to 28 of the
  imports of every program (`imports-*` artifacts): Rust's standard library
  needs `WaitOnAddress`, `ProcessPrng`, `CreateWaitableTimerExW`,
  `GetFileInformationByHandleEx`, and more, and winit, cpal, rfd, and dirs
  need touch, DWM, `mmdevapi.dll`, and known folders.  ReactOS keeps its
  Vista functions in a private `kernel32_vista.dll`, so even Rust's tier 3
  `win7` targets fail.  On 0.4.16 the loader refuses every `*_ST.EXE` with
  `c0000139`, "The procedure entry point AcquireSRWLockExclusive could not
  be located in the dynamic link library KERNEL32.dll".
- **With the shim, every program loads** (0 missing imports on all three
  images), and on 0.4.16 the console programs exit 0, the threads' TLS
  destructors run, iced's counter draws correctly through tiny-skia, and
  cinq opens its main window (`vm` artifact: `results.txt`, `reactos.mp4`,
  `shots.tar`).  cinq has no sound there, since ReactOS has no WASAPI.
- On `windows-latest`, the same programs with the shim behave as without
  it, for both processors (`windows` artifact).

The x64 nightlies are MSVC debug builds that wait for WinDbg on COM1 from
the first boot of setup, so only the i386 release ran in the VM.

## What the shim and the patch work around

- Imports that ReactOS lacks: the shim defines `__imp_X` for each, which
  looks for the function at run time in its DLL, then in that DLL's
  `*_vista.dll`, and otherwise falls back, for example to
  `RtlGenRandom` for `ProcessPrng` and to polling for `WaitOnAddress`.
- A loader bug: ReactOS's `LdrShutdownThread` runs a thread's FLS callbacks
  only after `LdrpFreeTls` has freed its TLS blocks (`dll/ntdll/ldr/
  ldrinit.c`, unchanged on master), while Windows runs them first.  Rust's
  standard library now runs TLS destructors from an FLS callback, so every
  thread that exited read and wrote freed memory: `RefCell` "already
  borrowed" panics in `destructors/list.rs:31` and the assertion
  `RtlTestBit(&Heap->FreeHintBitmap, HintIndex)` in `heap.c`.  The shim
  emulates FLS with TLS slots and runs the callbacks from the program's
  own TLS callback, before the loader frees TLS.
- Stubs in `ucrtbase.dll`: `trunc`, `truncf`, `log1p`, `acosh`, `asinh`,
  and `cbrt`, double and float, are exported stubs that pass the loader
  and return nothing useful.  cosmic-text places glyphs with `truncf`, so
  iced drew all text at the top of the window until the shim replaced
  them.  The C programs and Windows drew it correctly throughout.
- cpal 0.18.2 unwraps `CoCreateInstance(MMDeviceEnumerator)` and panics
  without WASAPI; `patches/cpal-0.18.2` returns no devices instead, which
  cinq takes as no sound.

wgpu found only OpenGL 1.1 in ReactOS and no adapter, so iced fell back to
tiny-skia, which draws through GDI.

Run 42469204654 (811b902), before the math and cpal fixes, showed the
counter with its text at the top of the window (`vm` artifact) and cinq
stopping at cpal's panic, and the same counter drawn correctly on Windows
(`windows` artifact, `counter-i686.png`).  Run 42470183373 was cancelled,
since 42470325528 holds its commit.
