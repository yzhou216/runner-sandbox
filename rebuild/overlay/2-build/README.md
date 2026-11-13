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
