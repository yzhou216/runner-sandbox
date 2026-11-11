# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""The message of every commit of rv64_emu, keyed by subject."""

MESSAGES = {}


def m(text):
    MESSAGES[text.split('\n', 1)[0]] = text.strip('\n') + '\n'


m('''treewide: initial commit

The project consists of a RISC-V emulator with its test harness, the C
and RISC-V assembly test programs that it runs, bit field helpers, and
the skeleton of an instruction cache simulator.  The emulator does not
decode any instruction yet.
''')
m('''rv_emu: emulate arithmetic, jalr and branch instructions

rv_one() dispatches on the opcode to the first instructions that the
test programs need:

  R-type  add, sub, mul, sll, srl, and
  I-type  addi, srli
  jalr
  B-type  blt, bne

Shift amounts have six bits, as on RV64.  jalr links the return address
into rd and jumps to rs1 plus its offset.  x0 is cleared after every
instruction, since writes to it are discarded.
''')
m('''rv_emu: emulate sb and sd

The offset is sign-extended, since it may be negative.
''')
m('rv_emu: emulate jal and j')
m('''rv_emu: emulate lb and ld

Both the loaded byte and the offset are sign-extended.
''')
m('rv_emu: emulate beq')
m('''rv_emu: emulate rem

The remainder is that of the signed values, with the results that
RISC-V defines for a zero divisor and for overflow, which C leaves
undefined.
''')
m('''rv_emu: emulate div

The quotient is that of the signed values, with the results that RISC-V
defines for a zero divisor and for overflow, which C leaves undefined.
''')
m('''rv_emu: emulate bge

The branch cases are now ordered by funct3.
''')
m('''rv_emu: emulate lw

The loaded word is sign-extended.
''')
m('''asm: spell out slli in sort_s

sort_s shifts by an immediate, which sll only accepts as an assembler
alias for slli.  Both forms assemble to the same instruction.
''')
m('''rv_emu: emulate slli

Like srli, slli takes a six-bit shift amount.
''')
m('rv_emu: emulate sw')
m('''rv_emu: drop the remaining typedefs

The remaining typedefs were rv_analysis and cache_type.  rv_emu.h now
uses block comments, and the prototypes in the headers no longer have a
space before their parameter lists.
''')
m('''treewide: rename cache structs and drop redundant braces

struct cache_st and struct cache_slot_st become struct cache and struct
cache_slot.  The remaining line comments become block comments, the
spacing around casts and pointer declarators is fixed, and a note
explains the padding of the '% Used' format string, which the expected
test output relies on.
''')
m('rv_emu: decide the branch outcome before updating pc')
m('rv_emu: count executed instructions for -a')
m('''cache: support four-word blocks in the direct-mapped cache

Instruction words are now fetched through cache_lookup(), and a
direct-mapped miss refills the whole block.  The set-associative cache
is still a stub.
''')
m('''asm: align the test functions to 16 bytes

16 bytes is the size of a four-word cache block.  The stray '#! riscv'
line in quadratic_s.s is also removed.
''')
m('''cache: use lru replacement in the set-associative cache

Misses also refill whole blocks, as in the direct-mapped cache, and the
word offset and the number of ways to search come from the address and
the configuration instead of being hard-coded.
''')
m('''rv_emu: tidy up rv_init

The spacing around casts is fixed, and the comment on the zero register
is reworded.
''')
m('cache: switch on the cache type in cache_lookup')
m('''cache: honor the configured block size

The word offset and the refill assumed four-word blocks.
''')
m('cache: factor block refill out of the lookup functions')
m('readme: describe the project')
m('''build: compile as c2y with a configurable toolchain

A cross toolchain can now be selected on the command line:

    make CC=riscv64-linux-gnu-gcc AS=riscv64-linux-gnu-as

-Wall and -Wextra reported that sort_c() returned int against its
prototype, that char_to_digit() fell off its end for characters that
are not digits, and that cache_lookup_dm() and str_to_int_test() had an
unused variable and parameter, all of which are fixed.  The test
programs are assembled with -march=rv64g, since the emulator only
decodes 32-bit instructions and an rv64gc default would compress them,
and git ignores the built program.
''')
m('''tests: add golden-output regression tests

The 371 cases cover every test program without a cache, with -a, and
with direct-mapped and 2-, 4-, and 8-way set-associative caches.  make
check runs them, RUN selects an emulator such as qemu-riscv64 for cross
builds, and a run that hangs is killed after TIMEOUT seconds.

The cache statistics of sort depend on how far apart the linker places
sort_s and find_max_index_s, a distance that depended on every C object
linked between them.  The assembly objects are therefore linked first
and next to each other.
''')
m('''treewide: adopt c23 features

  - bool, true, and false as keywords instead of <stdbool.h>
  - nullptr instead of NULL
  - constexpr objects instead of constant macros, including DEBUG, so
    that the debug print is an if statement rather than a conditional
    block
  - enums with fixed underlying types for the ABI registers and the
    instruction encodings, which replace the last typedef, with digit
    separators that group the opcode bits as the opcode map of the
    specification does
  - [[noreturn]], [[nodiscard]], [[unsequenced]], and [[gnu::format]]
    on verbose(), which exposed uint64_t values printed with %d and %X,
    now printed with %w64u and %w64X
  - empty initializers instead of memset() and per-field resets, which
    also initialize the emulator state in main()

OP instructions are dispatched on funct7 and funct3 together, which
turns the nested if-else chains in emu_r_type() into one switch, and an
unsupported one is reported by its whole instruction word.  The
encodings live in rv_emu.c, since only the emulator decodes
instructions.
''')
m('''rv_emu: align the emulated stack to 16 bytes

The RISC-V calling convention keeps sp 16-byte aligned, but the stack
array only had the 8-byte alignment of the registers before it, so the
emulated programs started with sp misaligned by 8.  The array is now
declared with the C23 alignas keyword, and a static_assert checks that
its size keeps the top of the stack aligned as well.
''')
m('''treewide: adopt c2y features

countof from <stdcountof.h> gives the lengths of arrays, a switch
statement in rv_one() declares the decoded instruction, and an if
statement in main() declares the selected test program.

The fields that all instruction formats share are decoded once into
struct rv_insn instead of again in every emulation function.  They are
unsigned _BitInt types as wide as the fields, so the conversions in
rv_decode() do the masking, and conversions to signed _BitInt types of
the right width sign-extend the immediates.  Bit-precise types are not
promoted, so funct7 is widened before it is shifted into the OP key.
Each emulation function now computes its result and writes it through
set_rd(), which discards writes to x0, so rv_one() no longer clears x0
afterward.  The functions are named after their major opcodes.

stdc_trailing_ones() from <stdbit.h> counts the index and offset bits
of the cache, and a table of test programs gives all of them one
signature.
''')
m('''cache: restructure the simulator

evict_data_in_cache_block() is now cache_fill_block(), since it fills a
slot rather than evicting anything, and its indentation is fixed.  Both
lookups return from a hit directly, cache_find_tag(),
cache_find_invalid(), and cache_find_lru() search a set, and an if
statement that declares the set-associative hit scopes it to its
branch.

The valid flag is now a bool.  A comment explains the padding of the
'% Used' line instead of blaming the test cases for it.
verbose_cache(), the slots_used field, and the verbose field of struct
rv_state are removed, since nothing used them, and g_verbose is declared
next to verbose().
''')
m('''treewide: tidy up the headers, prototypes and parameters

The headers have include guards.  The C test programs include
rv64_emu.h, so their definitions are checked against the prototypes
through which the harness calls them, and the four that disagreed are
corrected.  Pointer parameters that are only read are const,
char_to_digit() classifies digits with C2y case ranges, the option loop
ends in its condition, and the continuation lines in rv_print() are
aligned.
''')
m('''tests: exercise instruction corner cases with insn_s

insn_s covers what the other test programs do not reach: loads and
stores through negative offsets, jalr through an odd offset and with rd
equal to rs1, shifts by 32 or more, and writes to x0.  It has no C
version, so its test compares the native results with the emulated
ones only, and it is linked after the other assembly programs so that
it does not move them.
''')
m('''rv64_emu: validate the command line and the cache geometry

A missing program name, a missing argument of a test program, or a
missing value of -dm or -sa made rv64_emu dereference a null argv entry
and crash.  int_to_str crashed unless -o was given, although it
defaults to base 10, and unknown options were ignored.  The table of
test programs now records their arguments and the range of their
argument counts, which main() checks in the condition of the if
statement that declares the program.  The option values are checked as
well, and any error prints the options and the programs to stderr.

cache_init() derives its masks on the assumption that the block size
and the number of sets are powers of two, and the slots and blocks have
fixed sizes, but nothing checked the sizes given to -dm and -sa.
cache_valid() rejects the geometries that the simulator cannot model,
using stdc_has_single_bit() for the powers of two, and the used slots
are counted over the slots instead of over the words.
''')
m('''treewide: give the cache and the logging their own headers

rv_emu.h declared the cache simulator and the logging functions next to
the emulator, so cache.c and verbose.c had to include the whole
emulator interface to get at their own declarations.  The cache types
and functions now live in cache.h and the logging in verbose.h, and
each file includes only what it uses.  The Makefile lists every header
as a dependency, including bits.h, which it missed before.
''')
m('''rv_emu: harden the emulator loop

rv_init() clears all registers instead of only the ones that it sets,
and emulation stops with an error when sp moves below the emulated
stack, before a store through it can overwrite the registers below.
Unsupported instructions are reported on stderr after stdout is
flushed, so that the results of earlier runs come out first, and
unsupported OP-IMM instructions are reported by their word, as OP ones
are.  The code to emulate is passed through a const pointer, and
get_bits() builds its mask from a 64-bit one, which no longer overflows
for 31 bits or more.
''')
m('''cache: simplify the lookups and report the usage in slots

cache_split() splits an address into its tag, index, and word offset in
one place, the dispatch returns the result of each lookup directly, the
evicted address that -v logs includes the block offset bits, and the
loops whose body is an if statement have braces.

cache_print() labeled the size as slots, although it counts words, and
divided the valid slots by that word count, so a full cache with
four-word blocks showed 25% used.  The size is now labeled as words,
and the valid slots are divided by the number of slots.
''')
m('''rv64_emu: check more arguments and tidy up the harness

int_to_str rejects bases other than 2, 10, and 16, since it loops
forever for bases 0 and 1.  to_upper rejects strings that do not fit
its buffer, and str_to_int rejects strings that are not numbers in
their base, on which the C and assembly versions disagree.  The -a flag
lives in rv64_emu.c instead of struct rv_state, the type of the cache
tells whether one is configured, the parameters of the test functions
are aligned, and the loop in find_test() has braces.
''')
m('''treewide: fix and tidy up the test programs

int_to_str_s terminated the string after the middle of the reversed
digits, which cut off every number with more than two digits, and
wrote no digit for zero.  It also divided the sign-extended register,
while int_to_str_c() converts an unsigned 32-bit value, and the two
wrote hexadecimal digits in different cases.  The digits are now
terminated before they are reversed, the value is zero-extended, and
both versions write uppercase digits.

get_bitseq_c() shifted the int constant 1 by the length of the
sequence, which overflows for 31 bits, and now shifts an unsigned one.
int_to_str_c() picks digit characters with case ranges, and all C test
programs follow the brace rules of the kernel coding style.
''')
m('readme: describe building and testing the c2y version')
