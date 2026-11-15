# Reading/writing game options

- You need this feature for all other features except baseline.
- To enable this feature and work on it, add `config` to the `enabled_features`
  list in your `Cargo.toml`.

## What you need to do

You need to modify `config.rs` in the following way:

1. Modify `GameConfig` (and some relevant types) so it can be serialized and
   deserialized using serde.
2. Implement `GameConfig::load`.
3. Create unit tests that test `load`.  Your tests should comprehensively test
   the following:
   - loading options for different bag types (only one of these will be here,
     and you can add the other two when implementing the `rng` feature).
   - loading options for `animate_title`.

Overall, I expect you to write 3 lines of implementation code for this feature.
So, most of your contribution will be writing tests.  Your tests should go into
a test module inside `config.rs` (next to my tests for `build_game_state`).

Notice that you are not given any tests for this feature itself.  This is for
making you practice writing tests.  So, **you** will write unit tests instead
for this feature.  Later features will rely on this feature being correct, so
you need to ensure and test correctness now in order to avoid latent bugs.
