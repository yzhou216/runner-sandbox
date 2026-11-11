# Hard drop

**This feature's tests are not ready yet.  I will update this file once it is
ready.**

To enable this feature and work on it, add `hard_drop` to the `enabled_features`
list in your `Cargo.toml`.

## What you need to do

You need to **create** new systems to implement the option to enable/disable
hard drop.  All of this code should go into `hard_drop.rs`.  You also need to
modify the `HardDropPlugin` to register these systems appropriately (see other
plugins in the game for examples of doing this).

The systems you create should do a few things:
- When the user presses the `Z` key, the hard drop flag must be flipped.
- Other systems should observe this change and update the hard drop status text,
  and the gravity value in the game state.
  
If you haven't implemented handling the input for down arrow using the manual
gravity value in the game state, then you will need to modify your the logic to
match that.  In this game, when the manual gravity value is N, we should
interpret it as dropping the block by 1 N times until there is a collision
(i.e., as if the relevant button is pressed N times).
