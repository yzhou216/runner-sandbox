//! Record-replay functionality used for testing.
#![warn(missing_docs)]

use std::{collections::VecDeque, time::Duration};

use bevy::{ecs::resource::Resource, input::keyboard::KeyCode};
use serde::{Deserialize, Serialize};

mod record;
mod replay;
pub mod test_replay;

#[allow(unused_imports)]
pub use record::*;

#[allow(unused_imports)]
pub use replay::*;

use crate::{board::Block, data::Tetromino};

/// A recording of the events and the game state.
#[derive(Serialize, Deserialize, Resource, Default)]
pub struct GameRecording {
    /// Input events, ordered by time
    pub events: VecDeque<InputEvent>,
    /// State snapshots, ordered by time, used for testing and debugging.
    pub snapshots: VecDeque<(Duration, Snapshot)>,
}

/// An input event (which keys are pressed/released, etc.) with a time stamp.
#[derive(Serialize, Deserialize)]
pub struct InputEvent {
    /// Time this event occurred at, as a duration since the startup
    pub time: Duration,
    /// Set of keys just pressed
    pub just_pressed: Vec<KeyCode>,
    /// Set of keys just released
    pub just_released: Vec<KeyCode>,
}

/// A snapshot of the game state for testing and debugging.
#[derive(Serialize, Deserialize, Debug)]
#[allow(missing_docs)]
pub struct Snapshot {
    active: Option<Tetromino>,
    next: Option<Tetromino>,
    /// Obstacle vector, ordered as a block
    obstacles: Vec<Block>,
    hold: Option<Tetromino>,
    hard_drop: bool,
    manual_gravity: u32,
    score: u32,
    lines_cleared: u32,
    level: u32,
}

impl PartialEq for Snapshot {
    fn eq(&self, other: &Self) -> bool {
        // this is divided to allow disabling features
        #[allow(unused_mut)]
        let mut result = self.active == other.active
            && self.next == other.next
            && self.obstacles == other.obstacles;

        #[cfg(feature = "hold")]
        {
            result = result && self.hold == other.hold;
        }

        #[cfg(feature = "hard_drop")]
        {
            result = result
                && self.hard_drop == other.hard_drop
                && self.manual_gravity == other.manual_gravity;
        }

        #[cfg(feature = "score")]
        {
            result = result
                && self.score == other.score
                && self.lines_cleared == other.lines_cleared
                && self.level == other.level;
        }

        result
    }
}

/// Fixed frame rate, to adjust timing for record and replay
pub const FIXED_FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 64);
