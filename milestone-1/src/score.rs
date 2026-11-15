//! Scoring subsystem

use crate::{Game, ui::mk_text};

use super::data::*;
use bevy::prelude::*;

#[derive(Component)]
struct ScoreMarker;

/// An event denoting that some lines are cleared
#[derive(Event)]
pub struct LinesCleared(pub u32);

/// Initialize the score text
fn setup_score_text(mut commands: Commands) {
    commands.spawn((
        crate::score::ScoreMarker,
        mk_text(
            "",
            Node {
                margin: auto().all().with_right(percent(10)),
                ..default()
            },
            None,
        ),
    ));
}

/// Score multiplier for clearing the given number of lines at once
const SCORE_MULTIPLIERS: [u32; 5] = [0, 40, 100, 300, 1200];

/// Update the game state when lines are cleared
fn update_score(event: On<LinesCleared>, mut state: ResMut<GameState>) {
    let lines_cleared = event.0;
    assert!(lines_cleared <= 4);

    // Score with the level from before the lines were cleared
    state.score += SCORE_MULTIPLIERS[lines_cleared as usize] * (state.level + 1);
    state.lines_cleared += lines_cleared;
    state.lines_cleared_since_last_level += lines_cleared;

    // Level up after (level + 1) * 10 lines, which also speeds up gravity
    if state.lines_cleared_since_last_level >= (state.level + 1) * 10 {
        state.level += 1;
        state.lines_cleared_since_last_level = 0;
        let interval = state.drop_interval();
        state.gravity_timer.set_duration(interval);
    }
}

/// Update the score text.
fn update_score_text(state: Res<GameState>, mut text: Single<&mut Text, With<ScoreMarker>>) {
    text.set_if_neq(Text::new(format!(
        "Score: {}\nLevel: {}\nLines: {}",
        state.score(),
        state.level(),
        state.lines_cleared
    )));
}

/// Plugin that adds scoring and the level system to the game.
pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_score_text.in_set(Game))
            .add_systems(Update, update_score_text.in_set(Game))
            .add_observer(update_score);
    }
}
