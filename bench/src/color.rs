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
