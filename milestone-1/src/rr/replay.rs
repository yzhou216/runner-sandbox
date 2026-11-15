//! Replaying a recorded gameplay

use super::*;
use bevy::{prelude::*, time::TimeUpdateStrategy};

fn replay_input(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut recording: ResMut<GameRecording>,
    time: Res<Time<Fixed>>,
) {
    if let Some(event) = recording
        .events
        .pop_front_if(|event| event.time <= time.elapsed())
    {
        for key in &event.just_released {
            keyboard.release(*key);
        }
        for key in &event.just_pressed {
            keyboard.press(*key);
        }
    }
}

/// A plugin for replaying a game recording.  The game recording must be given
/// as a `GameRecording` resource.
pub struct ReplayPlugin;

impl Plugin for ReplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPreUpdate, replay_input)
            .insert_resource(TimeUpdateStrategy::ManualDuration(FIXED_FRAME_DURATION));
    }
}
