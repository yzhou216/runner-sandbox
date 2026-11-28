// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Turning the pictures of a track upright, as the matrix in its header
//! tells
//!
//! A phone that films on its side writes the picture as the sensor sees it
//! and a matrix that turns it by a right angle, which the picture shows
//! with.  Only turns by right angles count, as those are what cameras write.

#[cfg(test)]
mod tests;

use mp4_atom::Matrix;

use super::Picture;

/// One whole turn in the fixed-point numbers of a matrix, 16 bits of which
/// follow the point
const ONE: i32 = 1 << 16;

/// Side of the square tiles in which the pixels of a picture move as it turns
const TILE: usize = 32;

/// How far a track turns its picture clockwise to show it upright
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::back::video) enum Turn {
    #[default]
    None,
    Quarter,
    Half,
    ThreeQuarters,
}

impl Turn {
    /// Returns the turn that `matrix` makes, or none for a matrix that is not
    /// a turn by a right angle
    pub(in crate::back::video) fn of(matrix: &Matrix) -> Self {
        match (matrix.a, matrix.b, matrix.c, matrix.d) {
            (0, ONE, c, 0) if c == -ONE => Self::Quarter,
            (a, 0, 0, d) if a == -ONE && d == -ONE => Self::Half,
            (0, b, ONE, 0) if b == -ONE => Self::ThreeQuarters,
            _ => Self::None,
        }
    }
}

impl Picture {
    /// Returns the picture turned clockwise by `turn`, as it is when there is
    /// no turn, so that most videos pay nothing for it.  The pixels move in
    /// square tiles, so that the rows that a quarter turn reads across stay in
    /// the cache: a 4K picture turns in about 7 ms, where a copy in the order
    /// of the turned picture took 45.
    pub(super) fn turned(self, turn: Turn) -> Self {
        if turn == Turn::None {
            return self;
        }
        let (width, height) = (self.width as usize, self.height as usize);
        let (across, down) = match turn {
            Turn::Quarter | Turn::ThreeQuarters => (height, width),
            Turn::None | Turn::Half => (width, height),
        };
        // Place in the turned picture of the pixel at `x` and `y`
        let target = |x: usize, y: usize| match turn {
            Turn::None => y * width + x,
            Turn::Quarter => x * height + (height - 1 - y),
            Turn::Half => (height - 1 - y) * width + (width - 1 - x),
            Turn::ThreeQuarters => (width - 1 - x) * height + y,
        };
        let (source, _) = self.pixels.as_chunks::<4>();
        let mut pixels = vec![[0; 4]; source.len()];
        (0..height)
            .step_by(TILE)
            .flat_map(|top| (0..width).step_by(TILE).map(move |left| (top, left)))
            .for_each(|(top, left)| {
                (top..(top + TILE).min(height)).for_each(|y| {
                    (left..(left + TILE).min(width))
                        .for_each(|x| pixels[target(x, y)] = source[y * width + x]);
                });
            });
        Self {
            width: across as u32,
            height: down as u32,
            pixels: pixels.into_flattened(),
        }
    }
}
