# Randomized bag

**This feature's tests are not ready yet.  I will update this file once it is
ready.**

To enable this feature and work on it, add `rng` to the `enabled_features`
list in your `Cargo.toml`.

## What you need to do

Implement the missing members of the `RandomizedBag` type in `bag.rs`.  To use a
fixed seed, you can use `SmallRng`'s `seed_from_u64` method.

The bag should start empty, and whenever it is queried for a piece and is empty,
it should refill itself.  Refilling the bag should create 1 tetromino of each
type (you can use `ALL_TETROMINO_TYPES`), randomize the order, and insert them
into the bag.  See the `rand` library's API for how to shuffle a list.  The way
you do shuffle has to match how I want it to be done (specified via test cases).
