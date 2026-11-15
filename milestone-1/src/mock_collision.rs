//! Fake version of collision code for initial prototyping.
#![allow(dead_code)]

use super::board::Block;
use super::data::*;
use bevy::prelude::*;

/// See collision::there_is_collision
pub fn there_is_collision(
    tetromino: &Tetromino,
    _obstacles: Query<&Block, With<Obstacle>>,
) -> bool {
    !tetromino.in_bounds()
}

/// See collision::delete_full_lines
pub fn delete_full_lines() {}
