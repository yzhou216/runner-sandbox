// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pictures that the decoder hands out, turned into RGBA with the colors of
//! the standard that their size implies
//!
//! H.264 holds a picture as the luma of each pixel and two differences of
//! color, which each square of four pixels shares, and a matrix turns them
//! into red, green, and blue.  High-definition video takes the matrix of
//! BT.709, and smaller video that of BT.601, both in their limited range.
//! The two pixels of each pair side by side share their differences, so
//! each pair converts at once.  The strips of rows of a high-definition
//! picture convert on threads of their own, so that it keeps up at sixty
//! pictures a second, while a smaller picture converts on the thread that
//! asks, since starting threads for each picture would cost it more than
//! they save.

#[cfg(test)]
mod tests;

use std::{num::NonZero, thread};

use rust_h264::decoder::Frame;

/// Lines from which a picture counts as high definition
const HIGH_DEFINITION: u32 = 720;

/// Most strips of rows that a high-definition picture converts in at once,
/// each on a thread of its own, enough to spread a picture over the cores
/// of most machines
const STRIPS: usize = 8;

/// Picture of a video, in RGBA, row by row from the top
pub struct Picture {
    pub width: u32,
    pub height: u32,
    /// Red, green, blue, and alpha of each pixel, a byte each
    pub pixels: Vec<u8>,
}

/// Parts that the differences of color of a pixel add to its red, green,
/// and blue, in 256ths of a step of luma for each step of difference
#[derive(Clone, Copy)]
struct Matrix {
    red_of_red: i32,
    green_of_blue: i32,
    green_of_red: i32,
    blue_of_blue: i32,
}

/// Matrix of BT.601, which smaller video takes
const BT601: Matrix = Matrix {
    red_of_red: 409,
    green_of_blue: -100,
    green_of_red: -208,
    blue_of_blue: 516,
};

/// Matrix of BT.709, which high-definition video takes
const BT709: Matrix = Matrix {
    red_of_red: 459,
    green_of_blue: -55,
    green_of_red: -136,
    blue_of_blue: 541,
};

/// Returns a decoded picture in RGBA, which holds its colors in eight bits
/// and shares them among four pixels
pub(super) fn rgba(picture: &Frame) -> Picture {
    let matrix = if picture.height < HIGH_DEFINITION {
        BT601
    } else {
        BT709
    };
    Picture {
        width: picture.width,
        height: picture.height,
        pixels: converted(picture, matrix),
    }
}

/// Returns the pixels of `picture` in RGBA with `matrix`, in limited range,
/// converting the strips of rows of a high-definition picture on threads of
/// their own
fn converted(picture: &Frame, matrix: Matrix) -> Vec<u8> {
    let (width, height) = (picture.width as usize, picture.height as usize);
    let mut pixels = vec![0; width * height * 4];
    if picture.height < HIGH_DEFINITION {
        converted_rows(picture, matrix, 0, &mut pixels);
    } else {
        let strips = thread::available_parallelism()
            .map_or(1, NonZero::get)
            .min(STRIPS);
        // Rows of a strip, in pairs, so that a strip shares no chroma with
        // the next
        let rows = height.div_ceil(strips).next_multiple_of(2).max(2);
        thread::scope(|scope| {
            pixels
                .chunks_mut(rows * width * 4)
                .enumerate()
                .for_each(|(strip, pixels)| {
                    scope.spawn(move || converted_rows(picture, matrix, strip * rows, pixels));
                });
        });
    }
    pixels
}

/// Converts the rows of `picture` from `first` on into `pixels`, which hold
/// as many rows in RGBA as they have room for, with `matrix`
fn converted_rows(picture: &Frame, matrix: Matrix, first: usize, pixels: &mut [u8]) {
    let width = picture.width as usize;
    let chroma_width = picture.chroma_width() as usize;
    pixels
        .chunks_exact_mut(width * 4)
        .zip(first..)
        .for_each(|(line, row)| {
            let lumas = &picture.y[row * width..][..width];
            let blues = &picture.u[row / 2 * chroma_width..];
            let reds = &picture.v[row / 2 * chroma_width..];
            line.chunks_mut(8)
                .zip(lumas.chunks(2))
                .zip(blues.iter().zip(reds))
                .for_each(|((pair, lumas), (&blue, &red))| {
                    let chroma = Chroma::new(matrix, blue, red);
                    pair.as_chunks_mut::<4>()
                        .0
                        .iter_mut()
                        .zip(lumas)
                        .for_each(|(pixel, &luma)| *pixel = chroma.rgba(luma));
                });
        });
}

/// Part that the two differences of color of a pair of pixels add to
/// their red, green, and blue, in 256ths
#[derive(Clone, Copy)]
struct Chroma {
    red: i32,
    green: i32,
    blue: i32,
}

impl Chroma {
    /// Returns the part that the differences `blue` and `red` add, with
    /// `matrix`
    #[inline(always)]
    fn new(matrix: Matrix, blue: u8, red: u8) -> Self {
        let (blue, red) = (i32::from(blue) - 128, i32::from(red) - 128);
        Self {
            red: matrix.red_of_red * red,
            green: matrix.green_of_blue * blue + matrix.green_of_red * red,
            blue: matrix.blue_of_blue * blue,
        }
    }

    /// Returns the RGBA of a pixel of luma `luma` with these differences
    #[inline(always)]
    fn rgba(self, luma: u8) -> [u8; 4] {
        let luma = 298 * (i32::from(luma) - 16);
        let channel = |value: i32| ((value + 128) >> 8).clamp(0, 255) as u8;
        [
            channel(luma + self.red),
            channel(luma + self.green),
            channel(luma + self.blue),
            255,
        ]
    }
}
