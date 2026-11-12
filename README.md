<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# rv64_emu: renames

This session folded a set of renames of variables, struct members, and
struct tags into every commit of master and c2y of
[yzhou216/rv64_emu](https://github.com/yzhou216/rv64_emu), so that each
name appears in its new form in the commit that introduces it.  The names
lost types (`rsp`, `csp`, `block_base_iw_ptr`, `cache_st`, `signed_imm`),
a scope prefix (`g_verbose`), and abbreviations (`iw`, `i_count`,
`b_index`, `numer`), among others.

[rebuild/rename.py](rebuild/rename.py) holds the renames, which
[rebuild/ctok.py](rebuild/ctok.py) applies to the identifiers of the
function that holds them, along with the comments that named them and the
lines that the longer names push past 80 columns.
[rebuild/rewrite.py](rebuild/rewrite.py) rewrote the history from a fresh
clone, and [rebuild/same-code.sh](rebuild/same-code.sh) built every
original commit and its rewritten counterpart with the RISC-V toolchain of
the flake, without hardening and without PIE, whose GOT follows the names
of the symbols, and compared their code and data.

## Runs

All on `ubuntu-latest`:

| Run | Result |
|-----|--------|
| [43195659927](https://github.com/yzhou216/runner-sandbox/actions/runs/43195659927) | Passed |

## Outcome

The rewrite reproduced the history that the laptop built: master
`6d758ff711e9eb4089e5c3ea4877e452c5d6e3d3`, with 28 commits, and c2y
`5d83c6211b5ef9a5feb9202445a1a15e2a0a0f08`, with 15 more.  All 43
rewritten commits compiled to the same `.text`, `.rodata`, `.data`,
`.sdata`, and `.srodata` as their originals, so the renames change no
behavior.  The laptop pushed both branches to rv64_emu.
