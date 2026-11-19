//! Hard drop implementation
use bevy::prelude::*;

use crate::ui::*;

/// Tracker for the hard drop status
#[derive(Component, Default)]
pub struct HardDrop(pub bool);

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

// TODO: create systems for:
// - handling the user input (toggle hard drop when Z is pressed)
// - updating the gravity value based on HardDrop
// - updating the status text based on HardDrop
// for efficiency, use `Changed<HardDrop>` in your queries when it makes sense.

pub struct HardDropPlugin;

impl Plugin for HardDropPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_status_text);
        todo!("add your systems here.  They should go in Update, and in the Game system set.")
    }
}
