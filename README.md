<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: the ReactOS port, run in ReactOS 0.4.16

Checked cinq's port to ReactOS, branch `reactos` of yzhou216/cinq at
0729168, which adds `src/reactos/` and `.#i686-windows`.  The `build` job
builds `github:yzhou216/cinq/<commit>#i686-windows` and `vm/runner.c` with
the same MinGW, `imports` lists what ReactOS 0.4.16 and the x86 nightly
0.4.17-dev-1340-gfaa2d03 lack of their imports, and `vm` installs ReactOS
0.4.16 unattended in QEMU with KVM, as the earlier branch
`cinq/2026-11-08-reactos` describes.  There `vm/run.cmd` runs cinq through
`RUNNER.EXE`, whose results go to COM1, and `vm/keys.sh` types into cinq's
window through QEMU's monitor: `o` to add a cue, `Control+s`, `show`, and
Enter to save the show, then `Control+o` and Escape.

## Outcome

Run 42497131564 (d5accc4) passed on `ubuntu-latest`:

- `CINQ.EXE` and `RUNNER.EXE` lack no imports on either image.
- `cinq --version` printed its version and exited 0.
- cinq's window drew correctly, `o` added an audio cue, `Control+s` showed
  the save dialog, which saved `show.cinq.toml` in My Documents and named
  the window after it, and `Control+o` showed the open dialog, which listed
  the show (`vm` artifact: `results.txt`, `reactos.mp4`, `shots.tar`).  cinq
  ran until stopped, after 150 seconds.

cinq's own CI on the same commit passed every workflow, including the
32-bit build on x86_64-linux and aarch64-linux, which `windows-latest`
started with `--version`.
