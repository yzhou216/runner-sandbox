// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the pictures of picture cues: reading their files, decoding
//! them onto black, and the video output showing them and making them
//! ready ahead

use std::{
    fs,
    path::{Path, PathBuf},
};

use image::{Rgb, RgbImage, Rgba, RgbaImage};
use tempfile::TempDir;

use super::{showing_cue, synced};
use crate::back::{
    Command, Cue, CueId, Kind, Video, Workspace,
    video::{
        decoder::Sent,
        probe_still,
        still::{self, NO_PICTURE},
    },
};

/// Returns a directory that holds `slide.png`, a picture four pixels wide
/// and two high, all of them of `color`, along with the path of the
/// picture
fn slide(color: Rgba<u8>) -> (TempDir, PathBuf) {
    let directory = tempfile::tempdir().expect("a directory can be made");
    let path = directory.path().join("slide.png");
    RgbaImage::from_pixel(4, 2, color)
        .save(&path)
        .expect("the picture can be written");
    (directory, path)
}

/// Returns a workspace in the directory of the picture at `path`, with one
/// list of one picture cue that shows it, whose file has loaded
fn showing(path: &Path) -> Workspace {
    showing_cue(
        path,
        Cue::new(CueId(1), "1", Kind::Picture, "Slide")
            .file("slide.png")
            .build(),
    )
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
    let (shown, picture) = synced(&mut video, showing(&path).apply(Command::Go { list: 0 }));
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
    let workspace = video.sync(showing(&path)).0;
    assert!(video.ready.contains_key(&CueId(1)));
    let (_, picture) = synced(&mut video, workspace.apply(Command::Go { list: 0 }));
    assert!(picture.is_some());
    assert!(video.ready.is_empty());
}
