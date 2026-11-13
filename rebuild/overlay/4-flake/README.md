<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: GPL-3.0-or-later
-->

# rv64_cpu

Single-cycle 64-bit RISC-V processor in Digital

The processor executes the subset of RV64IM that its test programs in
RISC-V assembly need, one instruction per clock cycle.  Matchers that
recognize each instruction address a ROM of control words, which drive
the datapath, and the instruction memory holds a ROM for every program.
The `unimp` at the end of a program raises DONE, which stops the clock.

## Running

Open `src/rv64_cpu.dig` in [Digital](https://github.com/hneemann/Digital),
set PROG to the number in the name of a program in `progs`, clear the
processor with CLR, then set EN and run the clock until DONE rises.  A0,
A1, and A2 show the argument registers, and the probes below the
datapath show the others.

## Building

On a RISC-V machine, `make` assembles the programs into the ROM images
that `progs/hex` keeps, and turns the control words in
`decode/decode.bin` into the image of the decoder ROM:

    make -C progs
    make -C decode

Elsewhere, point `AS` and `OBJDUMP` at a RISC-V toolchain:

    make -C progs AS=riscv64-linux-gnu-as OBJDUMP=riscv64-linux-gnu-objdump

Digital stores what a ROM holds in the circuit, so a new image only
takes effect once it is loaded into its ROM.

With Nix, `nix run` opens the processor in Digital, and `nix develop`
opens a shell with Digital and the RISC-V binutils of Nixpkgs, in which
plain `make` builds the images.  `nix flake check` runs every program on
the processor and checks that the images and the ROMs match their
sources:

    nix run
    nix flake check

## License

rv64_cpu is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free
Software Foundation, either version 3 of the License, or any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
