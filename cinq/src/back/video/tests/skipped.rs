// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests that a video file whose tags or tracks other than its picture
//! cannot be read plays, and that one whose picture cannot be read does not

use std::fs;

use super::{
    boxes::{ilst_udta, narrow, reworked, split},
    damaged::{DAMAGED_INDEX, bytes, with_udta},
};
use crate::back::video::track::probe;
use tempfile::TempDir;

/// How long the test video plays, and that it holds no sound
const PLAYS: (f32, bool) = (5.0 / 30.0, false);

/// Probes the video that `bytes` make
fn probed(bytes: Vec<u8>) -> Result<(f32, bool), String> {
    let directory = TempDir::new().expect("a temporary directory can be made");
    let path = directory.path().join("skipped.mp4");
    fs::write(&path, bytes).expect("the video can be written");
    probe(&path)
}

/// Returns the boxes in the body of the box of `kind` in `bytes`
fn inside(bytes: &[u8], kind: &[u8; 4]) -> Vec<([u8; 4], Vec<u8>)> {
    let (_, body) = split(bytes)
        .into_iter()
        .find(|(found, _)| found == kind)
        .expect("the box is there");
    split(&body)
}

/// Returns `bytes` with `boxes` after the tables of the movie
fn with_boxes(bytes: &[u8], boxes: &[u8]) -> Vec<u8> {
    reworked(bytes, &[b"moov"], &|body| [body, boxes.to_vec()].concat())
}

/// Returns a trak box like that of the test video, with a media handler of
/// `handler` that its tables are too damaged to read: its stbl box is not
fn lost_track(handler: &[u8; 4]) -> Vec<u8> {
    let (_, trak) = inside(&bytes(), b"moov")
        .into_iter()
        .find(|(kind, _)| kind == b"trak")
        .expect("the movie has a track");
    let handled = reworked(
        &narrow(&(*b"trak", trak)),
        &[b"trak", b"mdia", b"hdlr"],
        &|body| [&body[..8], handler, &body[12..]].concat(),
    );
    renamed(&handled, b"stbl")
}

/// Returns `bytes` with the box of `kind`, which a box of another type
/// takes the place of, so that nothing reads it as the box that it was
fn renamed(bytes: &[u8], kind: &[u8; 4]) -> Vec<u8> {
    let at = bytes
        .windows(4)
        .position(|window| window == kind)
        .expect("the box is there");
    [&bytes[..at], b"skip", &bytes[at + 4..]].concat()
}

/// Returns `bytes` with `track` in place of the track of its movie
fn with_only_track(bytes: &[u8], track: &[u8]) -> Vec<u8> {
    reworked(bytes, &[b"moov"], &|body| {
        split(&body)
            .iter()
            .flat_map(|part| {
                if part.0 == *b"trak" {
                    track.to_vec()
                } else {
                    narrow(part)
                }
            })
            .collect()
    })
}

#[test]
fn tags_that_cannot_be_read_do_not_stop_the_video_from_playing() {
    // Nero's chapter list, one chapter at the start whose name is two bytes
    // that no text holds, after version 1, flags, and a reserved word
    let list = [&[1, 0, 0, 0][..], &[0; 4], &[1], &[0; 8], &[2, 0xff, 0xfe]].concat();
    let tags = [
        narrow(&(*b"meta", vec![0; 6])),
        narrow(&(*b"chpl", list)),
        [ilst_udta(), ilst_udta()].concat(),
        ilst_udta(),
    ];
    tags.iter()
        .for_each(|udta| assert_eq!(probed(with_udta(&bytes(), udta)), Ok(PLAYS)));
}

#[test]
fn box_that_claims_more_bytes_than_the_moov_box_holds_ends_the_walk_there() {
    // A free box that claims a megabyte, and a sound track after it that
    // the walk therefore never reaches, so that the file holds no sound
    let free = [&(1_u32 << 20).to_be_bytes()[..], b"free"].concat();
    let past = [free, lost_track(b"soun")].concat();
    assert_eq!(probed(with_boxes(&bytes(), &past)), Ok(PLAYS));
}

#[test]
fn tracks_that_cannot_be_read_leave_the_video_playing() {
    let plays = |handler: &[u8; 4], sound| {
        let with = with_boxes(&bytes(), &lost_track(handler));
        assert_eq!(probed(with), Ok((PLAYS.0, sound)), "{handler:?}");
    };
    // Chapters, a second picture, and sound, which still counts as sound
    plays(b"text", false);
    plays(b"vide", false);
    plays(b"soun", true);
}

#[test]
fn video_track_that_cannot_be_read_is_refused_for_its_index() {
    let lost = lost_track(b"vide");
    let refused = probed(with_only_track(&bytes(), &lost));
    assert_eq!(refused, Err(DAMAGED_INDEX.to_owned()));
    // And so is a track that cannot be read whose handler cannot either,
    // since it may be the picture
    let unnamed = with_only_track(&bytes(), &renamed(&lost, b"hdlr"));
    assert_eq!(probed(unnamed), Err(DAMAGED_INDEX.to_owned()));
}
