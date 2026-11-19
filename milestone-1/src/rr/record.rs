//! Recording functionality

use crate::data::{Active, GameState, Next, Obstacle};
#[cfg(feature = "hard_drop")]
use crate::hard_drop::HardDrop;
#[cfg(feature = "hold")]
use crate::hold::Hold;

use super::*;
use bevy::{prelude::*, time::TimeUpdateStrategy};
use std::path::PathBuf;

/// The file path to save the game recording to.
#[derive(Resource)]
pub struct RecordingOutput(pub PathBuf);

fn record_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut recording: ResMut<GameRecording>,
    time: Res<Time<Fixed>>,
) {
    // ignore the Escape key
    let just_pressed = keyboard
        .get_just_pressed()
        .cloned()
        .filter(|k| *k != KeyCode::Escape)
        .collect::<Vec<_>>();
    let just_released = keyboard
        .get_just_released()
        .cloned()
        .filter(|k| *k != KeyCode::Escape)
        .collect::<Vec<_>>();

    if just_pressed.is_empty() && just_released.is_empty() {
        return;
    }

    let elapsed = time.elapsed();

    recording.events.push_back(InputEvent {
        time: elapsed,
        just_pressed,
        just_released,
    });
}

/// Capture the game state at this point.
pub fn record_game_state(
    active: Option<Single<&Tetromino, With<Active>>>,
    next: Option<Single<&Tetromino, With<Next>>>,
    #[cfg(feature = "hold")] hold: Option<Single<&Tetromino, With<Hold>>>,
    obstacles: Query<&Block, With<Obstacle>>,
    #[cfg(feature = "hard_drop")] hard_drop: Single<&HardDrop>,
    state: Res<GameState>,
) -> Snapshot {
    let mut obstacles = obstacles.iter().copied().collect::<Vec<Block>>();
    obstacles.sort_by_key(|b| b.cell);

    Snapshot {
        active: active.map(|s| **s),
        next: next.map(|s| **s),
        #[cfg(feature = "hold")]
        hold: hold.map(|s| **s),
        #[cfg(not(feature = "hold"))]
        hold: None,
        obstacles,
        #[cfg(feature = "hard_drop")]
        hard_drop: hard_drop.0,
        #[cfg(not(feature = "hard_drop"))]
        hard_drop: false,
        manual_gravity: state.manual_drop_gravity,
        score: state.score(),
        lines_cleared: state.lines_cleared,
        level: state.level(),
    }
}

fn save_game_state(
    active: Option<Single<&Tetromino, With<Active>>>,
    next: Option<Single<&Tetromino, With<Next>>>,
    #[cfg(feature = "hold")] hold: Option<Single<&Tetromino, With<Hold>>>,
    obstacles: Query<&Block, With<Obstacle>>,
    #[cfg(feature = "hard_drop")] hard_drop: Single<&HardDrop>,
    state: Res<GameState>,
    time: Res<Time<Fixed>>,
    mut recording: ResMut<GameRecording>,
) {
    let snapshot = record_game_state(
        active,
        next,
        #[cfg(feature = "hold")]
        hold,
        obstacles,
        #[cfg(feature = "hard_drop")]
        hard_drop,
        state,
    );

    // Record only changed states
    if let Some((_, last)) = recording.snapshots.back()
        && snapshot == *last
    {
        return;
    }

    recording.snapshots.push_back((time.elapsed(), snapshot));
}

fn write_to_file(
    exit: MessageReader<AppExit>,
    recording: Res<GameRecording>,
    output: Res<RecordingOutput>,
) {
    if !exit.is_empty() {
        std::fs::write(&output.0, serde_json::to_vec(recording.as_ref()).unwrap())
            .expect("Cannot write to the given recording file");
    }
}

/// A plugin to enable recording to a file.  The file path must be given as a
/// `RecordingOutput` resource.
pub struct RecordingPlugin;

impl Plugin for RecordingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameRecording::default())
            .add_systems(FixedPreUpdate, record_keyboard)
            .add_systems(FixedPostUpdate, (save_game_state, write_to_file).chain())
            .insert_resource(TimeUpdateStrategy::ManualDuration(FIXED_FRAME_DURATION));
    }
}
