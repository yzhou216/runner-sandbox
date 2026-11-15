# Holding a piece

**This feature's tests are not ready yet.  I will update this file once it is
ready.**

To enable this feature and work on it, add `hold` to the `enabled_features` list
in your `Cargo.toml`.

## What you need to implement

Everything in `hold.rs`, see the relevant functions' documentation for the spec.

### Collision resolution

When a new piece is swapped in, if it is not in an illegal position, you need to
try moving it up (up to 4 times) until the collision is resolved.  If there is
still collision or the piece is out of bounds, then you should cancel/abort the
swap.

Note: this means that you shouldn't eagerly pop the next piece!
