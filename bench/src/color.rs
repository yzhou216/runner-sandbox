// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Conversion of decoded pictures to RGBA, taken from cinq's
//! src/back/video/decoder/color.rs at ec61ead, with the ways of spreading
//! it over threads that the benchmark compares

use std::{num::NonZero, sync::LazyLock, thread};

use rayon::prelude::*;
use rust_h264::decoder::Frame;

/// Ways of spreading a conversion over threads
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Way {
    /// On the calling thread alone
    Serial,
    /// Strips on scoped threads spawned for each picture, as cinq does for
    /// high definition
    Scoped,
    /// Same strips on rayon's pool
    RayonStrips,
    /// Bands of 16 rows on rayon's pool, which balances them
    RayonRows,
}

impl Way {
    pub const ALL: [Self; 4] = [Self::Serial, Self::Scoped, Self::RayonStrips, Self::RayonRows];

    pub fn name(self) -> &'static str {
        match self {
            Self::Serial => "serial",
            Self::Scoped => "scoped",
            Self::RayonStrips => "rayon-strips",
            Self::RayonRows => "rayon-rows",
        }
    }
}

/// Rows of each band of [`Way::RayonRows`]
const BAND: usize = 16;

pub static STRIPS: LazyLock<usize> = LazyLock::new(|| {
    thread::available_parallelism()
        .map_or(1, NonZero::get)
        .min(8)
});

#[derive(Clone, Copy)]
struct Matrix {
    red_of_red: i32,
    green_of_blue: i32,
    green_of_red: i32,
    blue_of_blue: i32,
}

const BT709: Matrix = Matrix {
    red_of_red: 459,
    green_of_blue: -55,
    green_of_red: -136,
    blue_of_blue: 541,
};

/// Returns the pixels of `picture` in RGBA, converted the way `way` tells
pub fn converted(picture: &Frame, way: Way) -> Vec<u8> {
    let (width, height) = (picture.width as usize, picture.height as usize);
    let mut pixels = vec![0; width * height * 4];
    convert_into(picture, way, &mut pixels);
    pixels
}

/// Converts `picture` into `pixels`, which hold room for all of it
pub fn convert_into(picture: &Frame, way: Way, pixels: &mut [u8]) {
    let (width, height) = (picture.width as usize, picture.height as usize);
    let matrix = BT709;
    let rows = height.div_ceil(*STRIPS).max(1);
    match way {
        Way::Serial => converted_rows(picture, matrix, 0, pixels),
        Way::Scoped => thread::scope(|scope| {
            pixels
                .chunks_mut(rows * width * 4)
                .enumerate()
                .for_each(|(strip, pixels)| {
                    scope.spawn(move || converted_rows(picture, matrix, strip * rows, pixels));
                });
        }),
        Way::RayonStrips => pixels
            .par_chunks_mut(rows * width * 4)
            .enumerate()
            .for_each(|(strip, pixels)| converted_rows(picture, matrix, strip * rows, pixels)),
        Way::RayonRows => pixels
            .par_chunks_mut(BAND * width * 4)
            .enumerate()
            .for_each(|(band, pixels)| converted_rows(picture, matrix, band * BAND, pixels)),
    }
}

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

/// Kernels that convert the rows of a picture, which all give the same
/// bytes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kernel {
    /// cinq's at ec61ead, a pixel at a time, as `converted_rows` does
    Now,
    /// Two pixels at a time, each packed into a word, and the pair stored
    /// at once
    Pairs,
    /// Sixteen pixels at a time in arrays, which LLVM vectorizes whole
    Lanes,
}

impl Kernel {
    pub const ALL: [Self; 3] = [Self::Now, Self::Pairs, Self::Lanes];

    pub fn name(self) -> &'static str {
        match self {
            Self::Now => "now",
            Self::Pairs => "pairs",
            Self::Lanes => "lanes",
        }
    }

    fn rows(self, picture: &Frame, first: usize, pixels: &mut [u8]) {
        match self {
            Self::Now => converted_rows(picture, BT709, first, pixels),
            Self::Pairs => pairs_rows(picture, BT709, first, pixels),
            Self::Lanes => lanes_rows(picture, BT709, first, pixels),
        }
    }
}

/// Returns the pixels of `picture` in RGBA, converted with `kernel` the
/// way `way` tells
pub fn converted_with(picture: &Frame, way: Way, kernel: Kernel) -> Vec<u8> {
    let (width, height) = (picture.width as usize, picture.height as usize);
    let mut pixels = vec![0; width * height * 4];
    let rows = height.div_ceil(*STRIPS).max(1);
    match way {
        Way::Serial => kernel.rows(picture, 0, &mut pixels),
        Way::Scoped => thread::scope(|scope| {
            pixels
                .chunks_mut(rows * width * 4)
                .enumerate()
                .for_each(|(strip, pixels)| {
                    scope.spawn(move || kernel.rows(picture, strip * rows, pixels));
                });
        }),
        Way::RayonStrips => pixels
            .par_chunks_mut(rows * width * 4)
            .enumerate()
            .for_each(|(strip, pixels)| kernel.rows(picture, strip * rows, pixels)),
        Way::RayonRows => pixels
            .par_chunks_mut(BAND * width * 4)
            .enumerate()
            .for_each(|(band, pixels)| kernel.rows(picture, band * BAND, pixels)),
    }
    pixels
}

impl Chroma {
    /// Returns the RGBA of a pixel of luma `luma` with these differences,
    /// packed into a word with red in its lowest byte
    #[inline(always)]
    fn word(self, luma: u8) -> u32 {
        let luma = 298 * (i32::from(luma) - 16) + 128;
        let channel = |value: i32| (value >> 8).clamp(0, 255) as u32;
        channel(luma + self.red)
            | channel(luma + self.green) << 8
            | channel(luma + self.blue) << 16
            | 0xff00_0000
    }
}

fn pairs_rows(picture: &Frame, matrix: Matrix, first: usize, pixels: &mut [u8]) {
    let width = picture.width as usize;
    let chroma_width = picture.chroma_width() as usize;
    pixels
        .chunks_exact_mut(width * 4)
        .zip(first..)
        .for_each(|(line, row)| {
            let lumas = &picture.y[row * width..][..width];
            let blues = &picture.u[row / 2 * chroma_width..][..width.div_ceil(2)];
            let reds = &picture.v[row / 2 * chroma_width..][..width.div_ceil(2)];
            let (pairs, last) = line.as_chunks_mut::<8>();
            let (luma_pairs, last_luma) = lumas.as_chunks::<2>();
            pairs
                .iter_mut()
                .zip(luma_pairs)
                .zip(blues.iter().zip(reds))
                .for_each(|((pair, &[left, right]), (&blue, &red))| {
                    let chroma = Chroma::new(matrix, blue, red);
                    let words = u64::from(chroma.word(left)) | u64::from(chroma.word(right)) << 32;
                    *pair = words.to_le_bytes();
                });
            if let (Some(&luma), Some(&blue), Some(&red)) =
                (last_luma.first(), blues.last(), reds.last())
            {
                last.copy_from_slice(&Chroma::new(matrix, blue, red).word(luma).to_le_bytes());
            }
        });
}

/// Pixels that [`Kernel::Lanes`] converts at once
const LANES: usize = 16;

fn lanes_rows(picture: &Frame, matrix: Matrix, first: usize, pixels: &mut [u8]) {
    let width = picture.width as usize;
    let chroma_width = picture.chroma_width() as usize;
    pixels
        .chunks_exact_mut(width * 4)
        .zip(first..)
        .for_each(|(line, row)| {
            let lumas = &picture.y[row * width..][..width];
            let blues = &picture.u[row / 2 * chroma_width..][..width.div_ceil(2)];
            let reds = &picture.v[row / 2 * chroma_width..][..width.div_ceil(2)];
            let (blocks, rest) = line.as_chunks_mut::<{ LANES * 4 }>();
            let (luma_blocks, _) = lumas.as_chunks::<LANES>();
            let (blue_blocks, _) = blues.as_chunks::<{ LANES / 2 }>();
            let (red_blocks, _) = reds.as_chunks::<{ LANES / 2 }>();
            blocks
                .iter_mut()
                .zip(luma_blocks)
                .zip(blue_blocks.iter().zip(red_blocks))
                .for_each(|((out, lumas), (blues, reds))| {
                    let lumas: [i32; LANES] = std::array::from_fn(|i| i32::from(lumas[i]));
                    let blues: [i32; LANES] = std::array::from_fn(|i| i32::from(blues[i / 2]) - 128);
                    let reds: [i32; LANES] = std::array::from_fn(|i| i32::from(reds[i / 2]) - 128);
                    let words: [u32; LANES] = std::array::from_fn(|i| {
                        let luma = 298 * (lumas[i] - 16) + 128;
                        let channel = |value: i32| (value >> 8).clamp(0, 255) as u32;
                        channel(luma + matrix.red_of_red * reds[i])
                            | channel(
                                luma + matrix.green_of_blue * blues[i] + matrix.green_of_red * reds[i],
                            ) << 8
                            | channel(luma + matrix.blue_of_blue * blues[i]) << 16
                            | 0xff00_0000
                    });
                    out.as_chunks_mut::<4>()
                        .0
                        .iter_mut()
                        .zip(words)
                        .for_each(|(pixel, word)| *pixel = word.to_le_bytes());
                });
            let done = blocks.len() * LANES;
            rest.as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(done..)
                .for_each(|(pixel, column)| {
                    let chroma = Chroma::new(matrix, blues[column / 2], reds[column / 2]);
                    *pixel = chroma.word(lumas[column]).to_le_bytes();
                });
        });
}
