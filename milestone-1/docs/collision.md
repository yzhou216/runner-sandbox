# Collision and full-line removal

**This feature's tests are not ready yet.  I will update this file once it is
ready.**

To enable this feature and work on it, add `collison` to the `enabled_features`
list in your `Cargo.toml`.

For this feature, you need to implement the functions in `collision.rs`.  See
those functions' documentation for the intended behavior.  Do not change the
functions under `mock_collision.rs`, those are provided as baseline tests.  If
you haven't implemented systems like `gravity` using on these functions, you may
need to fix those systems.

After you implement these properly, all tests should pass.

## A note about bounds checking

Although there are 20 rows visible in the game, there are 3 additional invisible
rows.  If a piece is completely within these rows (visible or invisible) then it
is in bounds.
