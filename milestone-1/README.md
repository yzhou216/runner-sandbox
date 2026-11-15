# Milestone 1: Single-player game

In this assignment, you will implement a simple Tetris-like game. Like the
future milestones, the milestone itself is divided into several features to
simulate a real software engineering scenario where you build and/or improve
upon a product by adding features over time.

While working on this assignment, you will:

1. Work on a repo over a longer term than most other courses.
2. Have a dialogue with other developers (your TAs and instructor) through code
   reviews, and improve your code.
3. Merge multiple features one-by-one, thus practice continuous integration.
4. Write your own tests for some of the features you will implement.
4. Become a better Rust programmer and game developer.

You need to _pass_ this assignment in order to qualify for a D or above, and in
order to move onto the next course.

## Deadlines

- Soft deadline: Friday, March 13 (right before the Spring break).
- Hard deadline: Monday, March 30.
    - This is the last day for you to **pass all code reviews** and schedule an
      interactive grading session.
    - Note: The gap between soft and hard deadlines will shrink for future
      assignments because there is a hard deadline imposed by the university at
      the end of the semester.

## Testing

For this assignment, we will use cargo-nextest because of some pecularities of
cargo-test.  For the curious (this is not relevant to CS 272): Bevy allows one
App per process and cargo-test runs each test in a separate thread in the same
process whereas cargo-nextest spawns child processes.

### Installing cargo-nextest

Run:

```
cargo install cargo-nextest --locked
```

You need to do this step only once for the whole semester.

### Running the tests

**If you are using macOS**, you have to use `cargo nt --test-threads=1`
because macOS cannot handle spawning tests off the main thread.  Unfortunately,
this means your tests will take longer to finish because of lack of parallelism.

If you are using other operating systems, you can just run `cargo nt`.

`nt` here is an alias. The full testing command is:

```
cargo nextestrun --features test --retries 3
```

Note that you will still need `--test-threads=1` if you're a macOS user.

## Directory structure

```
.
├── Cargo.lock
├── Cargo.toml             # Project configuration
├── README.md
├── src
│   ├── bag.rs             # RNG bag
│   ├── bin
│   │   └── blox.rs        # main program
│   ├── board.rs           # core systems and board logic
│   ├── collision.rs       # collision logic
│   ├── config.rs          # load/store configurations and generate states
│   ├── data.rs            # core ECS data structures
│   ├── hard_drop.rs       # hard drop feature
│   ├── hold.rs            # hold feature
│   ├── lib.rs             # the root.  contains the code that initializes the app
│   ├── mock_collision.rs  # placeholder until you implement collision
│   ├── score.rs           # score + level logic
│   └── ui.rs              # base UI
└── tests
```

I will release the full test suite in parts (releasing tests for sets of
features each week), be on the lookout for that.  Those tests will be locked
behind Cargo features.

## Features you need to implement

To isolate different aspects/features of this project, we will use Cargo features.
You can see the setup for the features in `Cargo.toml`.  Once you start working
on a feature, you need to add it to the list of features under `enabled_features`.

The features you need to implement (in the suggested order of implementation) are:

1. The baseline (no additional features enabled): Porting the work from the labs
   and implementing: spawning a new piece, rotation, and gravity.
   - This will also involve displaying the next piece, and making sure that the
     whole UI for the base game works.
2. `config`: Reading/writing game configurations using `serde`.
3. `collision`: Handling collision between different pieces and a game over
screen.
   - This is the point in which you will implement deleting full lines as well.
   - This is the first point where many tests will be able to work so you
     might have some latent bugs in earlier features that won't be revealed
     until this point.
4. `score`: Scoring and level system.
5. `rng`: Randomizing the bag, along with a configuration option for choosing
which bag implementation to use.
6. `hard_drop`: Implementing hard drop as an in-game option.
7. `hold`: Implementing the option to hold a piece.
8. `extra_credit`: You don't need to do this to pass the assignment, but you can
   do it to get a minor grade bump (e.g., B -> B+, C+ -> B-).

See `Cargo.toml` for the dependencies for each feature.  Each feature will have
a specification under `docs/<feature>.md`.  I will release the specs for
features beyond the first two when I release the relevant tests.

**Do not change the `dev` or `ci` features or add extra dependencies.  Doing so
might break the CI configuration.**

## Implementing and submitting a feature

You need to work on each feature in a separate branch (you can do so
concurrently if you don't want to wait for code reviews), and create 1 PR per
feature.  Each feature needs to have the following workflow:

1. Implement the feature on its own branch and make sure it passes all tests.
   - Write additional tests if the feature spec requires you to do so.
   - *Commit and push regularly.*
2. Create a PR for the feature.  **All your code and PR description must be
   professional.** In your PR description, give an overview of the structure
   and behavior of your code, and any tests you have added.
   - GitHub will assing a reviewer automatically.
3. Make sure the code passes the tests on the CI server too.
4. Go through the code review process until you get approval.
5. Merge the feature.

## Architecture of the game

See `board.rs` and `data.rs` for the core systems.  Here are a few differences
from the labs:

- There is a `Block` component to denote singular blocks that become obstacles
  and/or are parts of a side board.
- The `LockdownTimer` resource keeps track of the time until a piece is locked.
- `GameState` is what `GameConfig` was in the previous game.  The name change is
  there to distinguish the fully-materialized state (`GameState`) and the
  configuration read from the user (`GameConfig`).
- There are a few new marker components: `Hold`, `Next`, `Obstacle`, and
  `ScoreMarker`.
- The next tile bag is now a trait with multiple implementations
  (`DeterministicBag` vs. `RandomBag`), see `bag.rs` when you are implementing
  the `rng` feature.
- There is a `HardDrop` component to keep track of whether HardDrop is enabled +
  to access the status text.
- `LinesCleared` is an event that is triggered when new lines are cleared.  It
  should also keep track of how many lines are cleared by that event.
  
### Systems and lifecycle

See `build_app` for how the game lifecycle is built.

Here are the core systems:

Startup:
- Same as the gravity lab.

FixedUpdate:
- `gravity`, `spawn_next_tetromino`: same as the gravity lab.  However, the
  gravity interval is determined differently from the lab.
- `deactivate_if_stuck`: this should convert the tetromino into a set of
  obstacles.  This behavior is **different from the gravity lab!**.
- `delete_full_lines`: only used when `collision` is enabled, removes obstacles
  on full lines and uses naive gravity to move the obstacles above.  Interacts
  with scoring too (so, you need to write code that uses the `cfg` attributes).
- `game_over_on_esc`: triggers the game over event when the Escape key is
  pressed.
  
Update:
- `handle_user_input`: executes user actions on arrow keys and space (only the
  actions that move the tetromino).
- `redraw_board`: same idea as the last time but now with obstacles.
- `redraw_side_board`: redraws the side board associated with its generic
  argument.
- `animate_title`: same as the gravity lab but whether to enable it depends on
  the game configuration.

Additionally, the following features have their own plugins that add more
systems.  See the relevant feature specifications:
- `ScorePlugin`
- `HoldPlugin`
- `HardDropPlugin`

Collision does not have a separate plugin because the functionality it provides
is useful in the base game as well.  Instead, when you enable the feature the
definitions of `there_is_collision` and `delete_full_lines` switch to your
implementation from the placeholder one.

Random bag is also handled via configurations and feature flags (see below),
there are no systems associated specifically with that feature.

### Use of Rust feature flags

Some modules and definitions are gated behind `cfg` attributes, so they become
available only when certain features are enabled.  You won't be able to (nor
will you need to) use those definitions until you are implementing the relevant
feature.

### Caveats

- Use `Time<Fixed>` rather than `Time`.  Using the former allows us to simulate
  time passing faster in tests.  Using `Time` directly or `Time<Virtual>` will
  cause jitters or strange looking test failures (like time not passing at all).
  
- Some tests are flaky because they are timing dependent.  I set up the testing
  commands to retry these automatically.  If a flaky test eventually passes, you
  can ignore errors from failing runs.  For example, if you see an output like
  this then all the tests eventually passed even if nextest reports some
  transient failures:
  
  ```
       Summary [   0.889s] 15 tests run: 15 passed (1 flaky), 0 skipped
  ```
  
## AI use policy

The following uses of generative AI tools are OK for this milestone:
- Debugging a particular compiler error,
- Generating test cases for an implementation (any AI-generated or other
  external code must have proper attribution, e.g. via a comment `//from:
  <source>`), and
- Asking an AI tool "how do I do this?" or "what does this part of the code
  do?".

All of the code you submit as part of your implementation (i.e., anything that
is not a test) must be produced directly by you.  The AI use policy will get
relaxed over time, this is to ensure that you build a solid baseline before
using such tools so that you can thrive in this class rather than getting stuck
later.

## Grading

To pass this assignment, you need to:

1. Pass all the tests for all features,
2. Have your code pass `cargo fmt --check` and `cargo clippy`.
3. Pass code review.
4. Pass interactive grading.

For the code review, you have to work on a **non-main** branch and open a PR
when you want to test your implementation against GitHub actions.

### Code review

After you are done with step 2, you need to request code review.  The hard
deadline for **this assignment only** is the deadline to request code reviews.
The hard deadline in future assignments will be the deadline for _passing_ code
reviews.

After you are assigned a reviewer, they will review your code and request
explanations and/or changes.  Once you satisfy all these, yuo pass the code
review.  After you are done witch each set of changes, you need to ping the
reviewer on GitHub to ask for another round of code review until they approve
your changes.

### Interactive grading

After passing the code reviews, you have to go through an interactive grading
session.  This will be a 30-minute interview (i.e., oral exam) where you (1)
explain how your code works, the design decisions you made, and what approaches
you tried have not worked and (2) do a live demo of your game.

If you fail interactive grading once, you can do it again with me.  If you fail
it twice, you fail the whole assignment.  The objective of the assignment is to
show that **you have developed a certain understanding** and interactive
grading is where you will be able to demonstrate that.

