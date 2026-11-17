<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# rv64_cpu: pipeline videos

This session recorded Digital running programs on the five-stage pipeline
of [yzhou216/rv64_cpu](https://github.com/yzhou216/rv64_cpu), on its
branch `pipeline` (dc46153), with [record/record.py](record/record.py) on
a 3840 by 2160 Xvfb display.  The three programs that exercise the
hazards ran at 0.8 seconds per cycle, so that each stage can be followed,
and the sort at 0.06 seconds per cycle.

[tools](tools) holds the scripts that generated the pipeline.
[tools/netlist.py](tools/netlist.py) measures the pins of every element
from the SVG export of Digital and joins them into nets through wires and
tunnels, [tools/gen.py](tools/gen.py) places elements and connects their
pins by tunnels, and [tools/build.py](tools/build.py) writes the stage
registers, the hazard unit, and `pipeline.dig`, or, with `MUTANT` set, a
pipeline with one hazard mechanism broken, against which the tests of
rv64_cpu must fail.

## Runs

All on `ubuntu-latest`:

| Run | Result |
|-----|--------|
| [44019530466](https://github.com/yzhou216/runner-sandbox/actions/runs/44019530466) | Recorded 24-forward, 25-load-use, 26-flush, and 22-sort |

## Outcome

Every program reached DONE with the result that it documents: 24-forward
with A0 6, A1 4, and A2 2, 25-load-use with 8, 7, and 3, 26-flush with 2,
3, and 11, and 22-sort with 0xC0000000A, 0x800000006, and 0x400000002.
The recorder checks DONE every five cycles, so it counted 15, 30, 25, and
560 cycles for programs that take 11, 28, 25, and 557.
