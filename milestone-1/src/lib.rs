//! The core of the game, as a library.
#![allow(clippy::type_complexity)]
#![warn(missing_docs)]

use bevy::{app::App, ecs::schedule::SystemSet};

pub mod bag;
pub mod board;
pub mod config;
pub mod data;
pub mod rr;
pub mod ui;

#[cfg(feature = "hard_drop")]
pub mod hard_drop;

#[cfg(feature = "hold")]
pub mod hold;

#[cfg(feature = "score")]
pub mod score;

// collision is somewhat central to the game so we have a fake collision
// implementation and a real one gated behind feature flags
#[cfg(feature = "collision")]
mod collision;
#[cfg(feature = "collision")]
use collision::*;

#[cfg(not(feature = "collision"))]
mod mock_collision;
#[cfg(not(feature = "collision"))]
use mock_collision::*;

/// A system set to denote the systems that belong to the game.  We use this so
/// that the input systems injected by integration tests do not run into a race
/// condition with the systems from the game.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct Game;

/// A system set used by tests to inject systems before the actual game systems.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PreGame;

/// A system set used by tests to inject systems after the actual game systems.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PostGame;

/// Inject the systems and plugins for this game into the app.
pub fn build_app(app: &mut App, cfg: config::GameConfig) {
    use bevy::prelude::*;
    use board::*;
    use data::Next;
    use ui::*;
    app.insert_resource(cfg.build_game_state())
        .add_systems(
            Startup,
            (setup_board, spawn_next_tetromino, setup_ui)
                .chain()
                .in_set(Game),
        )
        .add_systems(
            FixedUpdate,
            (
                gravity,
                deactivate_if_stuck,
                delete_full_lines,
                spawn_next_tetromino,
                game_over_on_esc,
            )
                .chain()
                .in_set(Game),
        )
        .add_systems(
            Update,
            (handle_user_input, redraw_board, redraw_side_board::<Next>).in_set(Game),
        );

    #[cfg(all(not(feature = "ci"), not(feature = "test")))]
    app.add_systems(Startup, setup_window);

    if cfg.animate_title {
        app.add_systems(Update, animate_title);
    }

    #[cfg(feature = "score")]
    app.add_plugins(score::ScorePlugin);

    #[cfg(feature = "hold")]
    app.add_plugins(hold::HoldPlugin);

    #[cfg(feature = "hard_drop")]
    app.add_plugins(hard_drop::HardDropPlugin);
}
