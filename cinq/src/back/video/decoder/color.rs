// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pictures that openh264 decodes, turned into RGBA with the colors of the
//! standard that their size implies
//!
//! H.264 holds a picture as the luma of each pixel and two differences of
//! color, which each square of four pixels shares, and a matrix turns them
//! into red, green, and blue.  High-definition video takes the matrix of
//! BT.709, while smaller video takes that of BT.601, which is the matrix
//! that openh264 converts with, so pictures 720 lines tall or taller
//! convert here instead, with BT.709 in its limited range.  The two pixels
//! of each pair side by side share their differences, so each pair
//! converts at once, and strips of rows convert on threads of their own,
//! so that high-definition video keeps up at sixty pictures a second.

#[cfg(test)]
mod tests;

use std::{num::NonZero, thread};

use openh264::{decoder::DecodedYUV, formats::YUVSource};

/// Lines from which a picture counts as high definition
const HIGH_DEFINITION: usize = 720;

/// Most strips of rows that a picture converts in at once, each on a thread
/// of its own, enough to spread a picture over the cores of most machines
const STRIPS: usize = 8;

/// Picture of a video, in RGBA, row by row from the top
pub struct Picture {
    pub width: u32,
    pub height: u32,
    /// Red, green, blue, and alpha of each pixel, a byte each
    pub pixels: Vec<u8>,
}

/// Returns a decoded picture in RGBA
pub(super) fn rgba(picture: &DecodedYUV<'_>) -> Picture {
    let (width, height) = picture.dimensions();
    let pixels = if height < HIGH_DEFINITION {
        bt601(picture)
    } else {
        bt709(picture)
    };
    // H.264 limits pictures to far fewer pixels a side than u32 counts
    Picture {
        width: width as u32,
        height: height as u32,
        pixels,
    }
}

/// Returns the pixels of `picture` in RGBA with the matrix of BT.601, as
/// openh264 converts them
fn bt601(picture: &DecodedYUV<'_>) -> Vec<u8> {
    let (width, height) = picture.dimensions();
    let mut pixels = vec![0; width * height * 4];
    picture.write_rgba8(&mut pixels);
    pixels
}

/// Returns the pixels of `picture` in RGBA with the matrix of BT.709, in
/// limited range, converting strips of its rows on threads of their own
pub(super) fn bt709(picture: &(impl YUVSource + Sync)) -> Vec<u8> {
    let (width, height) = picture.dimensions();
    let strips = thread::available_parallelism()
        .map_or(1, NonZero::get)
        .min(STRIPS);
    // Rows of a strip, in pairs, so that a strip shares no chroma with the
    // next
    let rows = height.div_ceil(strips).next_multiple_of(2).max(2);
    let mut pixels = vec![0; width * height * 4];
    thread::scope(|scope| {
        pixels
            .chunks_mut(rows * width * 4)
            .enumerate()
            .for_each(|(strip, pixels)| {
                scope.spawn(move || bt709_rows(picture, strip * rows, pixels));
            });
    });
    pixels
}

/// Converts the rows of `picture` from `first` on into `pixels`, which hold
/// as many rows in RGBA as they have room for
fn bt709_rows(picture: &impl YUVSource, first: usize, pixels: &mut [u8]) {
    let (width, _) = picture.dimensions();
    let (luma_stride, blue_stride, red_stride) = picture.strides();
    pixels
        .chunks_exact_mut(width * 4)
        .zip(first..)
        .for_each(|(line, row)| {
            let lumas = &picture.y()[row * luma_stride..][..width];
            let blues = &picture.u()[row / 2 * blue_stride..];
            let reds = &picture.v()[row / 2 * red_stride..];
            line.chunks_mut(8)
                .zip(lumas.chunks(2))
                .zip(blues.iter().zip(reds))
                .for_each(|((pair, lumas), (&blue, &red))| {
                    let chroma = Chroma::bt709(blue, red);
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
    /// the matrix of BT.709
    #[inline(always)]
    fn bt709(blue: u8, red: u8) -> Self {
        let (blue, red) = (i32::from(blue) - 128, i32::from(red) - 128);
        Self {
            red: 459 * red,
            green: -55 * blue - 136 * red,
            blue: 541 * blue,
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
