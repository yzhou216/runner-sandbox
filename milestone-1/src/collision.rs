//! Collision detection implementation

use super::board::Block;
use super::data::*;
use bevy::{platform::collections::HashMap, prelude::*};

/// Return whether the given tetromino collides with any of the obstacles or it
/// is out of bounds.
pub fn there_is_collision(tetromino: &Tetromino, obstacles: Query<&Block, With<Obstacle>>) -> bool {
    !tetromino.in_bounds()
        || obstacles
            .iter()
            .any(|block| tetromino.cells().contains(&block.cell))
}

/// A system to detect the full lines (lines containing only obstacles and no
/// empty space) and to delete them.
///
/// After the lines are deleted, any obstacles above the deleted lines should be
/// moved down using naive gravity (the obstacles move down only by the number
/// of lines below them that are deleted).
pub fn delete_full_lines(
    mut commands: Commands,
    mut obstacles: Query<(Entity, &mut Block), With<Obstacle>>,
) {
    // Number of obstacles on each line
    let line_sizes = obstacles
        .iter()
        .fold(HashMap::<i32, u32>::new(), |mut sizes, (_, block)| {
            *sizes.entry(block.cell.1).or_default() += 1;
            sizes
        });
    let full_lines: Vec<i32> = line_sizes
        .into_iter()
        .filter(|&(_, size)| size == BOARD_WIDTH)
        .map(|(line, _)| line)
        .collect();
    if full_lines.is_empty() {
        return;
    }

    obstacles.iter_mut().for_each(|(entity, mut block)| {
        if full_lines.contains(&block.cell.1) {
            commands.entity(entity).despawn();
        } else {
            // Naive gravity: drop by the number of deleted lines below the block
            let line = block.cell.1;
            block.cell.1 -= full_lines.iter().filter(|&&full| full < line).count() as i32;
        }
    });

    #[cfg(feature = "score")]
    commands.trigger(crate::score::LinesCleared(full_lines.len() as u32));
}
