# Baseline implementation

- This feature is needed for all other features.
- You don't need to modify `Cargo.toml` for this feature at all.


## Before you implement anything

Read `blox.rs`, `lib.rs`, `data.rs` and `board.rs`.  There are subtle
differences from previous labs in terms of the data structures, and some new
ideas like side boards and obstacles.  If you do not understand the data model
fully, please ask questions on Ed.

Also, see the definition of `TestBag`, it is a mock bag used in testing to
create specific scenarios without relying on picking the "just right" random
seed.

## What you need to implement

1. Everything missing in `data.rs`.
2. The following in `board.rs`:
   - `LockdownTimer::start_or_advance`.
   - `handle_user_input`: see below
   - `gravity`: pretty much the same as the gravity lab.
   - `deactivate_if_stuck`: **do not just delete a tetromino, make sure that the
     relevant obstacles are also created!**
   - `spawn_next_tetromino`: similar to the gravity lab.
     - Make sure to update the next piece window too.
   - `redraw_board`: you also need to color in the obstacles.  Make sure to
   - `redraw_side_board`: this should be a simpler version of `redraw_board`.
     Do not change the parameter types, and do not add any new parameters.

## Handling user input

General rules:

- If an input would position a tetromino in an illegal configuration (out of
  bounds or collides with an obstacle), do not perform the update.  You can use
  `there_is_collision` for this.  You can create the new tetromino, do this
  check and update the actual component after the check.
- If multiple keys are pressed, both actions happen.  This is the order in which
  the actions must happen: down, left, right, up/space.
- Pressing up and space together should trigger only one rotation.

Inputs:
- down: Manually drop the tetromino.  The maximum amount of drops is determined
  by the game state (see `GameState`).  If you implement this properly now, the
  hard drop feature will be easy later.
- left, right: Move the tetromino in the specified the direction by 1 space.
- up or space: Rotate the tetromino.
