<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# rv64_cpu: history rebuild

This session checked the history of
[yzhou216/rv64_cpu](https://github.com/yzhou216/rv64_cpu), a single-cycle
64-bit RISC-V processor in Digital, which the laptop rebuilt from the
private classroom repository of project06 with
[rebuild/rebuild.py](rebuild/rebuild.py).  The script takes the files of
every new commit from an original one, moved into the layout of
rv64_cpu, folds the fix of the store unit and two debugging probes into
the commits that they belong to, drops the commits that only cleaned
up, and adds the commits from 2026 in [rebuild/overlay](rebuild/overlay):
the README, the build variables, REUSE compliance, the flake, and CI.
[rebuild/digdiff.py](rebuild/digdiff.py) and
[rebuild/wirediff.py](rebuild/wirediff.py) compare circuits by their
components and wires, which is how the folds were checked.

[rebuild/matrix.sh](rebuild/matrix.sh) runs every test program on the
processor of every commit in Digital, with the test cases that
[rebuild/progtests.py](rebuild/progtests.py) writes, and compares the
results with [rebuild/matrix.expected](rebuild/matrix.expected), which
the laptop recorded.  The commit that adds the flake, which has no CI
yet, got the nix flake check that CI runs on the commit after it.

## Runs

| Run | Runners | Result |
|-----|---------|--------|
| [43344392400](https://github.com/yzhou216/runner-sandbox/actions/runs/43344392400) | `ubuntu-latest`, `ubuntu-24.04-arm` | Passed |
| [43343263766](https://github.com/yzhou216/rv64_cpu/actions/runs/43343263766) | CI of rv64_cpu on `ubuntu-latest` and `ubuntu-24.04-arm` | Passed |

## Outcome

Every commit of master `81d61f4d4e99aa54e3c7e91a3d944cc6d2b8fad8` gave
the recorded results: each program passes from the commit that adds the
instructions that it needs and loads it, sb and lb from the commit that
adds them, since the store fix is folded into the data memory, and all
24 from the commit that decodes slli and srli by funct6 on.  The commit
that adds the flake passed nix flake check on x86_64 and aarch64, and so
did the last commit in the CI of rv64_cpu.
