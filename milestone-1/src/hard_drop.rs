//! Hard drop implementation
use bevy::prelude::*;

use crate::Game;
use crate::board::handle_user_input;
use crate::data::{GameState, HARD_DROP_GRAVITY, SOFT_DROP_GRAVITY};
use crate::ui::*;

/// Tracker for the hard drop status
#[derive(Component, Default)]
pub struct HardDrop(pub bool);

/// Show the hard drop status on the screen, with hard drop off.
pub fn setup_status_text(mut commands: Commands) {
    commands.spawn(mk_text(
        "Hard Drop: Off",
        (
            Node {
                width: percent(40),
                height: percent(100),
                row_gap: px(10),
                ..default()
            },
            HardDrop(false),
            UiTransform::from_translation(Val2::percent(10.0, 50.0)),
        ),
        None,
    ));
}

/// Flip the hard drop flag when Z is pressed.
fn toggle_hard_drop(keyboard: Res<ButtonInput<KeyCode>>, mut hard_drop: Single<&mut HardDrop>) {
    if keyboard.just_pressed(KeyCode::KeyZ) {
        hard_drop.0 = !hard_drop.0;
    }
}

/// Make the manual drop gravity follow the hard drop flag.
fn update_manual_gravity(
    hard_drop: Single<&HardDrop, Changed<HardDrop>>,
    mut state: ResMut<GameState>,
) {
    state.manual_drop_gravity = if hard_drop.0 {
        HARD_DROP_GRAVITY
    } else {
        SOFT_DROP_GRAVITY
    };
}

/// Show whether hard drop is on in the status text.
fn update_status_text(mut status: Single<(&mut Text, &HardDrop), Changed<HardDrop>>) {
    let (text, hard_drop) = &mut *status;
    text.0 = format!("Hard Drop: {}", if hard_drop.0 { "On" } else { "Off" });
}

/// Plugin that lets the player turn hard drop on and off.
pub struct HardDropPlugin;

impl Plugin for HardDropPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_status_text).add_systems(
            Update,
            // Toggle before the same frame's drop input is handled
            (toggle_hard_drop, update_manual_gravity, update_status_text)
                .chain()
                .before(handle_user_input)
                .in_set(Game),
        );
    }
}
