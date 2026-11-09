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
//! each pair converts at once, into a word of eight bytes that goes out in
//! one store, which lets the compiler convert many pairs in each vector
//! instruction; byte by byte, the conversion took four times as long on
//! x86-64, and twice as long on aarch64.  The strips of rows of a
//! high-definition picture convert on threads of their own, so that it keeps
//! up at sixty pictures a second, while a smaller picture converts on the
//! thread that asks, since starting threads for each picture would cost it
//! more than they save.

#[cfg(test)]
mod tests;

use std::{num::NonZero, sync::LazyLock, thread};

use rust_h264::decoder::Frame;

/// Width and lines from either of which a picture counts as high
/// definition, so that a wide picture cropped from one of 1280 by 720, such
/// as one of 1280 by 534, does as well
const HIGH_DEFINITION: (u32, u32) = (1280, 720);

/// Strips of rows that a high-definition picture converts in at once, each
/// on a thread of its own: one for each core, up to eight, enough to spread
/// a picture over the cores of most machines.  The cores are counted once,
/// since std counts them anew at each call, too slowly for every picture.
static STRIPS: LazyLock<usize> = LazyLock::new(|| {
    thread::available_parallelism()
        .map_or(1, NonZero::get)
        .min(8)
});

/// Picture in RGBA, a byte for each channel, row by row from the top
pub struct Picture {
    pub width: u32,
    pub height: u32,
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

const BT601: Matrix = Matrix {
    red_of_red: 409,
    green_of_blue: -100,
    green_of_red: -208,
    blue_of_blue: 516,
};

const BT709: Matrix = Matrix {
    red_of_red: 459,
    green_of_blue: -55,
    green_of_red: -136,
    blue_of_blue: 541,
};

/// Returns a decoded picture in RGBA, which holds its colors in eight bits
/// and shares them among four pixels
pub(super) fn rgba(picture: &Frame) -> Picture {
    let matrix = if high_definition(picture) {
        BT709
    } else {
        BT601
    };
    Picture {
        width: picture.width,
        height: picture.height,
        pixels: converted(picture, matrix),
    }
}

fn high_definition(picture: &Frame) -> bool {
    let (width, lines) = HIGH_DEFINITION;
    picture.width >= width || picture.height >= lines
}

fn converted(picture: &Frame, matrix: Matrix) -> Vec<u8> {
    let (width, height) = (picture.width as usize, picture.height as usize);
    let mut pixels = vec![0; width * height * 4];
    // Chunks of a picture that a crafted stream crops to no width would
    // panic
    if pixels.is_empty() {
        return pixels;
    }
    if high_definition(picture) {
        let rows = height.div_ceil(*STRIPS).max(1);
        thread::scope(|scope| {
            pixels
                .chunks_mut(rows * width * 4)
                .enumerate()
                .for_each(|(strip, pixels)| {
                    scope.spawn(move || converted_rows(picture, matrix, strip * rows, pixels));
                });
        });
    } else {
        converted_rows(picture, matrix, 0, &mut pixels);
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
            // H.264 crops a picture whose colors four pixels share to an
            // even width, so that each row is made of whole pairs
            line.as_chunks_mut::<8>()
                .0
                .iter_mut()
                .zip(lumas.as_chunks::<2>().0)
                .zip(blues.iter().zip(reds))
                .for_each(|((pair, &[left, right]), (&blue, &red))| {
                    let chroma = Chroma::new(matrix, blue, red);
                    let words = u64::from(chroma.rgba(left)) | u64::from(chroma.rgba(right)) << 32;
                    *pair = words.to_le_bytes();
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
    #[inline(always)]
    fn new(matrix: Matrix, blue: u8, red: u8) -> Self {
        let (blue, red) = (i32::from(blue) - 128, i32::from(red) - 128);
        Self {
            red: matrix.red_of_red * red,
            green: matrix.green_of_blue * blue + matrix.green_of_red * red,
            blue: matrix.blue_of_blue * blue,
        }
    }

    /// Returns the RGBA of a pixel of luma `luma` with these differences, as
    /// a word whose bytes from the lowest are red, green, blue, and alpha
    #[inline(always)]
    fn rgba(self, luma: u8) -> u32 {
        let luma = 298 * (i32::from(luma) - 16) + 128;
        let channel = |value: i32| (value >> 8).clamp(0, 255) as u32;
        channel(luma + self.red)
            | channel(luma + self.green) << 8
            | channel(luma + self.blue) << 16
            | 0xff << 24
    }
}
