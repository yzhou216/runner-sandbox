// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Video files that the tests make as they run, with the H.264 that
//! [`stream`] writes and the MP4 boxes of [`atoms`], so that the repository
//! holds none
//!
//! Each picture of a test video is gray, and lighter than the one before
//! it, so that a picture tells which one it is by its shade.

mod atoms;
mod stream;

use std::{fs, path::PathBuf};

use tempfile::TempDir;

use stream::{MACROBLOCKS, STEP, picture};

/// Width and height of a test video, in pixels
const SIZE: (usize, usize) = (MACROBLOCKS.0 * 16, MACROBLOCKS.1 * 16);

/// Pictures a second of a test video
pub(in crate::back) const RATE: u32 = 30;

/// Units of time a second in the track of a test video
const TIMESCALE: u32 = 600;

/// Shade of gray of the luma of the first picture, and how much lighter
/// each picture is than the one before it
const SHADES: (usize, usize) = (24, STEP);

/// Returns a temporary directory holding `movie.mp4`, a test video of
/// `count` pictures, at most 25, that decoding can start at every
/// `sync_every` pictures, with the path of the video
pub(in crate::back) fn movie(count: usize, sync_every: usize) -> (TempDir, PathBuf) {
    broken_movie(count, sync_every, &[])
}

/// Returns a test video as [`movie`] does, but whose pictures numbered in
/// `broken` hold bytes that no decoder can read
pub(in crate::back) fn broken_movie(
    count: usize,
    sync_every: usize,
    broken: &[usize],
) -> (TempDir, PathBuf) {
    saved(&movie_bytes(count, sync_every, broken))
}

/// Returns a temporary directory holding `movie.mp4`, a file of `bytes`,
/// with the path of the file
pub(super) fn saved(bytes: &[u8]) -> (TempDir, PathBuf) {
    let directory = TempDir::new().expect("a temporary directory can be made");
    let path = directory.path().join("movie.mp4");
    fs::write(&path, bytes).expect("the video can be written");
    (directory, path)
}

/// Returns which picture of a test video `pixels` show, by the shade of
/// their first pixel, which RGBA holds in limited range
pub(in crate::back) fn picture_number(pixels: &[u8]) -> usize {
    let (first, step) = SHADES;
    let luma = f64::from(pixels[0]) * 219.0 / 255.0 + 16.0;
    ((luma - first as f64) / step as f64).round().max(0.0) as usize
}

/// Returns the bytes of the file of [`broken_movie`]
pub(super) fn movie_bytes(count: usize, sync_every: usize, broken: &[usize]) -> Vec<u8> {
    let (first, step) = SHADES;
    let samples: Vec<Vec<u8>> = (0..count)
        .map(|number| {
            let unit = picture(number, sync_every, (first + step * number) as u8);
            avcc(&if broken.contains(&number) {
                garbled(&unit)
            } else {
                unit
            })
        })
        .collect();
    atoms::file(&samples, sync_every)
}

/// Returns a NAL unit with all but its header replaced by bytes that read
/// as no slice, and that hold no start code either
fn garbled(unit: &[u8]) -> Vec<u8> {
    let header = unit.first().copied().unwrap_or_default();
    [vec![header], vec![0x55; unit.len().saturating_sub(1)]].concat()
}

/// Returns a NAL unit after its length in four bytes, as an MP4 sample
/// holds it
fn avcc(unit: &[u8]) -> Vec<u8> {
    let length = u32::try_from(unit.len()).expect("units are small");
    [&length.to_be_bytes()[..], unit].concat()
}
