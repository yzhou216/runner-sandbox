<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# rv64_cpu: pipeline drawn with wires

This session recorded Digital running programs on the five-stage pipeline
of [yzhou216/rv64_cpu](https://github.com/yzhou216/rv64_cpu), on its
branch `pipeline`, after the pipeline was redrawn so that wires join its
parts instead of tunnels, with no two wires crossing.  The recorder in
[record/record.py](record/record.py) ran on a 3840 by 2160 Xvfb display,
the three programs that exercise the hazards at 0.8 seconds per cycle, and
the sort at 0.06 seconds per cycle.

[tools](tools) holds the scripts that generated the circuits.
[tools/build.py](tools/build.py) writes the stage registers, the hazard
unit, and `pipeline.dig`, or, given the name of a mutant, a pipeline with
one hazard mechanism broken, against which the tests of rv64_cpu must
fail.  [tools/gen.py](tools/gen.py) places elements, draws wires through
given points, and splits each wire where another ends on it, since
Digital joins wires only at their end points, and
[tools/check.py](tools/check.py) reports wires that cross.
[tools/planarity](tools/planarity) models the netlist as a graph, in which
every part is a wheel of its pins in clockwise order, and finds which nets
must stay tunnels for the drawing to have no crossings.

## Runs

All on `ubuntu-latest`:

| Run | Commit of rv64_cpu | Result |
|-----|--------------------|--------|
| [44293736359](https://github.com/yzhou216/runner-sandbox/actions/runs/44293736359) | af23f3b | Recorded 24-forward, 25-load-use, 26-flush, and 22-sort |
| [44294686389](https://github.com/yzhou216/runner-sandbox/actions/runs/44294686389) | d3ab015 | Recorded them again after A0, A1, and A2 moved below the register file |

## Outcome

In both runs every program reached DONE with the result that it
documents: 24-forward with A0 6, A1 4, and A2 2, 25-load-use with 8, 7,
and 3, 26-flush with 2, 3, and 11, and 22-sort with 0xC0000000A,
0x800000006, and 0x400000002.  The recorder checks DONE every five cycles,
so it counted 15, 30, 25, and 560 cycles for programs that take 11, 28,
25, and 557, as before the redraw.  In the first run the values of A0, A1,
and A2 overlapped beside the register file, which the second run no
longer shows.
