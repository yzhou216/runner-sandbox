//! Collision detection implementation

use super::board::Block;
use super::data::*;
use bevy::prelude::*;

/// Number of lines that can hold obstacles, including the invisible ones
const LINES: usize = (BOARD_HEIGHT + INVISIBLE_ROWS) as usize;

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
        .fold([0u32; LINES], |mut sizes, (_, block)| {
            sizes[block.cell.1 as usize] += 1;
            sizes
        });
    let full = line_sizes.map(|size| size == BOARD_WIDTH);
    let lines_cleared = full.iter().filter(|&&is_full| is_full).count() as u32;
    if lines_cleared == 0 {
        return;
    }

    // Naive gravity: the obstacles on each line drop by the number of deleted
    // lines below it, which is a cumulative sum from the bottom line up
    let drops: Vec<i32> = full
        .iter()
        .scan(0, |deleted_below, &is_full| {
            let drop = *deleted_below;
            *deleted_below += is_full as i32;
            Some(drop)
        })
        .collect();

    obstacles.iter_mut().for_each(|(entity, mut block)| {
        let line = block.cell.1 as usize;
        if full[line] {
            commands.entity(entity).despawn();
        } else {
            block.cell.1 -= drops[line];
        }
    });

    #[cfg(feature = "score")]
    commands.trigger(crate::score::LinesCleared(lines_cleared));
}
