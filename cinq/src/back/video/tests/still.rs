// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the pictures of picture cues, from their files to the video
//! output

use std::{fs, path::PathBuf};

use image::{Rgb, RgbImage, Rgba, RgbaImage};
use tempfile::TempDir;

use super::{GO, showing, synced};
use crate::back::{
    Command, CueId, Kind, Video,
    video::{
        decoder::Sent,
        probe_still,
        still::{self, NO_PICTURE, TOO_LARGE},
    },
};

fn slide(color: Rgba<u8>) -> (TempDir, PathBuf) {
    let directory = tempfile::tempdir().expect("a directory can be made");
    let path = directory.path().join("slide.png");
    RgbaImage::from_pixel(4, 2, color)
        .save(&path)
        .expect("the picture can be written");
    (directory, path)
}

#[test]
fn probe_reads_png_and_jpeg_and_tells_why_other_files_cannot_show() {
    let (directory, path) = slide(Rgba([255, 0, 0, 255]));
    assert_eq!(probe_still(&path), Ok(()));
    let photo = directory.path().join("photo.jpg");
    RgbImage::from_pixel(4, 2, Rgb([0, 0, 255]))
        .save(&photo)
        .expect("the photograph can be written");
    assert_eq!(probe_still(&photo), Ok(()));
    // Format goes by the first bytes of the file rather than its name
    let misnamed = directory.path().join("photo.png");
    fs::copy(&photo, &misnamed).expect("the photograph can be copied");
    assert_eq!(probe_still(&misnamed), Ok(()));
    let notes = directory.path().join("notes.png");
    fs::write(&notes, "not a picture").expect("the file can be written");
    assert_eq!(probe_still(&notes), Err(NO_PICTURE.to_owned()));
    let missing = directory.path().join("missing.png");
    assert_eq!(probe_still(&missing), Err("file not found".to_owned()));
}

#[test]
fn picture_lies_on_black_where_it_is_not_wholly_opaque() {
    let (_directory, path) = slide(Rgba([255, 128, 0, 128]));
    let sent: Vec<Sent> = still::start(path).iter().collect();
    let [Sent::Picture(frame), Sent::Ended] = sent.as_slice() else {
        panic!("the picture comes, and then the end of its pass");
    };
    assert_eq!(
        (frame.at, frame.picture.width, frame.picture.height),
        (0.0, 4, 2)
    );
    assert_eq!(frame.picture.pixels[..4], [128, 64, 0, 255]);
}

#[test]
fn video_output_shows_the_picture_of_a_picture_cue_until_it_stops() {
    let (_directory, path) = slide(Rgba([0, 255, 0, 255]));
    let mut video = Video::default();
    let (shown, picture) = synced(&mut video, showing(&path, Kind::Picture).apply(GO));
    let (cue, picture) = picture.expect("the picture comes due");
    assert_eq!((cue, picture.width, picture.height), (CueId(1), 4, 2));
    assert_eq!(picture.pixels[..4], [0, 255, 0, 255]);
    let (shown, pictures) = video.sync(shown.apply(Command::Tick(60.0)));
    assert!(pictures.is_empty());
    assert_eq!(video.runs.len(), 1);
    video.sync(shown.apply(Command::Stop(CueId(1))));
    assert!(video.runs.is_empty());
}

#[test]
fn picture_that_the_next_go_shows_is_made_ready_ahead() {
    let (_directory, path) = slide(Rgba([0, 0, 255, 255]));
    let mut video = Video::default();
    let workspace = video.sync(showing(&path, Kind::Picture)).0;
    assert!(video.ready.contains_key(&CueId(1)));
    let (_, picture) = synced(&mut video, workspace.apply(GO));
    assert!(picture.is_some());
    assert!(video.ready.is_empty());
}

/// Returns the PNG `png` with its header declaring a picture of `width` by
/// `height` pixels, and the checksum of the header made right
fn declared(png: &[u8], width: u32, height: u32) -> Vec<u8> {
    let header = [
        &png[12..16],
        &width.to_be_bytes(),
        &height.to_be_bytes(),
        &png[24..29],
    ]
    .concat();
    let checksum = !header.iter().fold(!0_u32, |crc, &byte| {
        (0..8).fold(crc ^ u32::from(byte), |crc, _| {
            (crc >> 1) ^ (0xEDB8_8320 * (crc & 1))
        })
    });
    [&png[..12], &header, &checksum.to_be_bytes(), &png[33..]].concat()
}

#[test]
fn picture_whose_header_declares_more_pixels_than_memory_holds_cannot_show() {
    let (_directory, path) = slide(Rgba([0, 0, 0, 255]));
    let png = fs::read(&path).expect("the picture can be read");
    fs::write(&path, declared(&png, 65535, 65535)).expect("the picture can be written");
    assert_eq!(probe_still(&path), Err(TOO_LARGE.to_owned()));
    // Decoding it asked the allocator for 16 GB, which ended cinq
    let sent: Vec<Sent> = still::start(path).iter().collect();
    assert!(
        matches!(sent.as_slice(), [Sent::Failed(reason)] if reason == TOO_LARGE),
        "the pass fails without a picture"
    );
}

#[test]
fn picture_of_the_most_pixels_that_fit_in_512_mebibytes_of_rgba_shows_and_one_row_more_cannot() {
    let (_directory, path) = slide(Rgba([0, 0, 0, 255]));
    let png = fs::read(&path).expect("the picture can be read");
    // 128 Mi pixels, which the manual calls 134 million
    fs::write(&path, declared(&png, 16384, 8192)).expect("the picture can be written");
    assert_eq!(probe_still(&path), Ok(()));
    fs::write(&path, declared(&png, 16384, 8193)).expect("the picture can be written");
    assert_eq!(probe_still(&path), Err(TOO_LARGE.to_owned()));
}
