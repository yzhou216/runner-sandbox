# Scoring and leveling system

**This feature's tests are not ready yet.  I will update this file once it is
ready.**

You will implement:
- Scoring based on lines cleared.
- Leveling up based on lines cleared.
- Updating the score text in the UI.
- Modifying your line clear function to update the gravity timer (if you haven't
  already).
  
Your changes for all parts except the last one should go into `score.rs`.
  
To work on this feature, add `score` to the `enabled_features` list in
Cargo.toml.

# The specification

When lines are cleared, two events happen: the score is increased and the level
might increase.

## Level increase

The level increases (current level + 1) * 10 lines are cleared since the last
level up.  For example, 10 lines -> level 1, 10 + 20 = 30 lines -> level 2.

## Score calculation

The score is increased by `score multiplier * (current level + 1)` whenever some
lines are cleared.

### Score multiplier table

| Lines cleared | Score multiplier |
|---------------|------------------|
| 1             | 40               |
| 2             | 100              |
| 3             | 300              |
| 4             | 1200             |

Clearing more than 4 lines at a time is not possible with the rules we are using.

## Updating the score text

See the initial score text.  You should update it the same way.
