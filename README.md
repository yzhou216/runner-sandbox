<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# rv64_emu: modular layout

This session folded a modular layout into every commit of the history of
[yzhou216/rv64_emu](https://github.com/yzhou216/rv64_emu), so that each
file exists in its final place from the commit that first needs it.  The
emulator lives in `src/emu`, split into the fetch loop, the decoder, the
dispatch, the integer, memory, and control transfer instructions, and the
dynamic analysis, the cache simulator in `src/cache`, the runs of the test
programs in `src/harness.c`, and the declarations of the programs in
`progs/progs.h`.

[rebuild/fold.py](rebuild/fold.py) maps the files of one original commit
to their rewritten counterparts, moving code verbatim between the
top-level chunks that [rebuild/ctop.py](rebuild/ctop.py) splits a file
into.  [rebuild/rewrite.py](rebuild/rewrite.py) rewrote the 43 commits up
to `5d83c6211b5ef9a5feb9202445a1a15e2a0a0f08` from a fresh clone, keeping
every author, committer, and date, and the messages of all but the five
commits that named code that moved.  [rebuild/compare.sh](rebuild/compare.sh)
built every original commit and its rewritten counterpart with the RISC-V
toolchain of the flake, ran the cases of `tests/cases` and of
`flake/agree.sh` under `qemu-riscv64` with both, and compared the output
and the compiler warnings.

[rebuild/cases.py](rebuild/cases.py) leaves out the cases whose output
depends on the layout of the C code in the original commit, which the
first comparison found and [rebuild/classify.py](rebuild/classify.py)
sorted by cause: cache geometries that the simulator cannot model before
the commit that validates them, the set-associative stub of commits 17
and 18, the slots that `cache_print()` read past the end of before that
commit, and the runaway `int_to_str` runs for bases 0 and 1 before the
commit that rejects them.

## Runs

All on `ubuntu-latest`, and the flake check on `ubuntu-24.04-arm` as well:

| Run | Result |
|-----|--------|
| [43304386639](https://github.com/yzhou216/runner-sandbox/actions/runs/43304386639) | Cancelled, since the rewrite added a blank line to a run of two |
| [43304724851](https://github.com/yzhou216/runner-sandbox/actions/runs/43304724851) | Differences in the cases that depend on the code layout |
| [43309633955](https://github.com/yzhou216/runner-sandbox/actions/runs/43309633955) | A segfault racing the timeout with its core dump in commit 36 |
| [43310777942](https://github.com/yzhou216/runner-sandbox/actions/runs/43310777942) | Passed |

## Outcome

The rewrite reproduced the history that the laptop built, ending in
`71888a18123195c492a4cc582829322dd86e23c5`.  Every rewritten commit built
like its original and printed the same output for every case compared,
407 of 417 before commit 37, 241 in commits 17 and 18, 416 in commits 37
to 40, and all 417 from commit 41 on.  The warnings matched, except that
the first commit warns that `rv_exec()` does not use `state`, since the
switch of the starter code has no case yet.  Commit 27, which adds the
flake before CI exists, passed `nix flake check` on x86_64 and aarch64.

CI of rv64_emu ran on commits 28 to 43, which have it, from temporary
branches that the laptop pushed.  All 16 runs passed, each building on
x86_64 and aarch64 and running the result on riscv64.  The laptop then
force-pushed master and c2y of rv64_emu to the last commit and deleted
the branches.

| Commit | Run |
|--------|-----|
| 28 `4c80865` | [43309691869](https://github.com/yzhou216/rv64_emu/actions/runs/43309691869) |
| 29 `98334fd` | [43309689314](https://github.com/yzhou216/rv64_emu/actions/runs/43309689314) |
| 30 `594907f` | [43309691781](https://github.com/yzhou216/rv64_emu/actions/runs/43309691781) |
| 31 `98b4cda` | [43309689053](https://github.com/yzhou216/rv64_emu/actions/runs/43309689053) |
| 32 `f423948` | [43309688066](https://github.com/yzhou216/rv64_emu/actions/runs/43309688066) |
| 33 `38c94bf` | [43309692767](https://github.com/yzhou216/rv64_emu/actions/runs/43309692767) |
| 34 `8ad07c1` | [43309690693](https://github.com/yzhou216/rv64_emu/actions/runs/43309690693) |
| 35 `f10a80b` | [43309687889](https://github.com/yzhou216/rv64_emu/actions/runs/43309687889) |
| 36 `d348ce2` | [43309688506](https://github.com/yzhou216/rv64_emu/actions/runs/43309688506) |
| 37 `2afca7d` | [43309692950](https://github.com/yzhou216/rv64_emu/actions/runs/43309692950) |
| 38 `48e21bd` | [43309692645](https://github.com/yzhou216/rv64_emu/actions/runs/43309692645) |
| 39 `8a1b92b` | [43309690628](https://github.com/yzhou216/rv64_emu/actions/runs/43309690628) |
| 40 `78286cc` | [43309690732](https://github.com/yzhou216/rv64_emu/actions/runs/43309690732) |
| 41 `499a35a` | [43309692100](https://github.com/yzhou216/rv64_emu/actions/runs/43309692100) |
| 42 `8cfe996` | [43309689939](https://github.com/yzhou216/rv64_emu/actions/runs/43309689939) |
| 43 `71888a1` | [43309691670](https://github.com/yzhou216/rv64_emu/actions/runs/43309691670) |
