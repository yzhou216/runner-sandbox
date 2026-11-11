# Extra Credit: Additional Features

To get the extra credit, you need to:

1. Implement the following features:
   - Wall kick
   - Shadow drop (ghost piece)
   - T-spin scoring
   - Music/SFX or auto-repeat
2. Have the tests for these features.
3. Go through code review (before hard deadline).
4. Demonstrate these features during interactive grading.

All of this code must be gated behind the `extra_credit` feature.

## Wall kick

You need to change how rotation and hold is handled by implementing the [wall
kick logic here](https://harddrop.com/wiki/SRS#Wall_kicks).

## Shadow drop (ghost piece)

Visualize exactly where the piece would eventually drop by drawing a
[silhouette](https://harddrop.com/wiki/Ghost_piece).  The exact visualization is
up to you.

## T-spin scoring

Detect if a [T-piece is rotated immediately before triggering a line
clear](https://harddrop.com/wiki/T-Spin).  It should also work with wall kicks.
Give + 25% score for this.

## Auto-repeat

Extend the configuration to take an auto-repeat rate (as frequency or seconds,
up to you).  When processing inputs, repeat a pressed directional input (left,
down, right but **not up**) according to the configuration.

## Music/SFX

Add background music and sound effects for line clear, and when a piece moves
down and touches an obstacle/the corners.  The details are up to you.
