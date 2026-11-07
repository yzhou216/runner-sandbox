// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Video files that the tests make as they run, with the H.264 that
//! [`stream`] writes and the MP4 writer of the mp4 crate, so that the
//! repository holds none
//!
//! Each picture of a test video is gray, and lighter than the one before
//! it, so that a picture tells which one it is by its shade.

mod stream;

use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

use mp4::{AvcConfig, Bytes, MediaConfig, Mp4Config, Mp4Sample, Mp4Writer, TrackConfig, TrackType};
use tempfile::TempDir;

use stream::{MACROBLOCKS, STEP, picture, picture_parameter_set, sequence_parameter_set};

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
pub(in crate::back) fn movie(count: usize, sync_every: u32) -> (TempDir, PathBuf) {
    broken_movie(count, sync_every, &[])
}

/// Returns a test video as [`movie`] does, but whose pictures numbered in
/// `broken` hold bytes that no decoder can read
pub(in crate::back) fn broken_movie(
    count: usize,
    sync_every: u32,
    broken: &[usize],
) -> (TempDir, PathBuf) {
    let directory = TempDir::new().expect("a temporary directory can be made");
    let path = directory.path().join("movie.mp4");
    write_movie(&path, count, sync_every as usize, broken);
    (directory, path)
}

/// Returns which picture of a test video `pixels` show, by the shade of
/// their first pixel, which RGBA holds in limited range
pub(in crate::back) fn picture_number(pixels: &[u8]) -> usize {
    let (first, step) = SHADES;
    let luma = f64::from(pixels[0]) * 219.0 / 255.0 + 16.0;
    ((luma - first as f64) / step as f64).round().max(0.0) as usize
}

/// Writes a test video to `path`, with the pictures numbered in `broken`
/// garbled
fn write_movie(path: &Path, count: usize, sync_every: usize, broken: &[usize]) {
    let (width, height) = SIZE;
    let avc = AvcConfig {
        width: width as u16,
        height: height as u16,
        seq_param_set: sequence_parameter_set(),
        pic_param_set: picture_parameter_set(),
    };
    let file = BufWriter::new(File::create(path).expect("the video file can be made"));
    let mut writer = Mp4Writer::write_start(file, &mp4_config()).expect("the video starts");
    writer
        .add_track(&TrackConfig {
            track_type: TrackType::Video,
            timescale: TIMESCALE,
            language: "und".to_owned(),
            media_conf: MediaConfig::AvcConfig(avc),
        })
        .expect("the track goes in");
    let duration = TIMESCALE / RATE;
    let (first, step) = SHADES;
    (0..count)
        .try_for_each(|number| {
            let unit = picture(number, sync_every, (first + step * number) as u8);
            let unit = if broken.contains(&number) {
                garbled(&unit)
            } else {
                unit
            };
            let sample = Mp4Sample {
                start_time: (number as u32 * duration).into(),
                duration,
                rendering_offset: 0,
                is_sync: number % sync_every == 0,
                bytes: Bytes::from(avcc(&unit)),
            };
            writer.write_sample(1, &sample)
        })
        .expect("the pictures go in");
    writer.write_end().expect("the video ends");
}

/// Returns a NAL unit with all but its header replaced by bytes that read
/// as no slice, and that hold no start code either
fn garbled(unit: &[u8]) -> Vec<u8> {
    let header = unit.first().copied().unwrap_or_default();
    [vec![header], vec![0x55; unit.len().saturating_sub(1)]].concat()
}

/// Returns the settings of the MP4 file of a test video
fn mp4_config() -> Mp4Config {
    let brand = |name: &str| name.parse().expect("brands are four characters");
    Mp4Config {
        major_brand: brand("isom"),
        minor_version: 512,
        compatible_brands: ["isom", "iso2", "avc1", "mp41"].map(brand).to_vec(),
        timescale: 1000,
    }
}

/// Returns a NAL unit after its length in four bytes, as an MP4 sample
/// holds it
fn avcc(unit: &[u8]) -> Vec<u8> {
    let length = u32::try_from(unit.len()).expect("units are small");
    [&length.to_be_bytes()[..], unit].concat()
}
