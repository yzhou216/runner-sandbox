// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of reading how long video files play and of decoding their
//! pictures in time, from the sync sample before a clip and on later passes
//! through a clip that loops, along with those of [`player`] and [`runs`],
//! on videos that [`movie`] makes, and those of [`still`], on pictures of
//! its own

pub(in crate::back) mod movie;
mod player;
mod runs;
mod still;

use std::{fs, path::Path, sync::Arc, time::Duration};

use super::{
    decoder::{self, Sent, UNDECODABLE},
    probe,
};
use crate::back::Clip;
use movie::{RATE, broken_movie, movie, picture_number};

/// Longest that a test waits for a thread to decode a picture
const PATIENCE: Duration = Duration::from_secs(10);

/// Returns the clip from `start` to `end` seconds into a file
fn clip(start: f64, end: f64, looping: bool) -> Clip {
    Clip {
        start,
        end,
        looping,
    }
}

/// Returns what decoding pass `pass` through `clip` of the video at `path`
/// sends, for a player that stays as long as the pass does
fn sent(path: &Path, clip: Clip, pass: u32) -> Vec<Sent> {
    let player = Arc::new(());
    decoder::start(path.to_owned(), clip, pass, Arc::downgrade(&player))
        .iter()
        .collect()
}

/// Returns the numbers of the pictures that decoding pass `pass` through
/// `clip` of the video at `path` sends, each with the time it is due at in
/// pictures of the video since the start of the clip, once the pass has
/// ended
fn decoded(path: &Path, clip: Clip, pass: u32) -> Vec<(usize, f64)> {
    let sent = sent(path, clip, pass);
    assert!(matches!(sent.last(), Some(Sent::Ended)), "the pass ends");
    sent.iter()
        .filter_map(|sent| match sent {
            Sent::Picture(frame) => Some((
                picture_number(&frame.picture.pixels),
                frame.at * f64::from(RATE),
            )),
            _ => None,
        })
        .collect()
}

/// Returns whether a picture decoded is the one expected, due at the time
/// expected but for rounding
fn same((number, at): (usize, f64), (expected, due): (usize, f64)) -> bool {
    number == expected && (at - due).abs() < 1e-6
}

/// Asserts that the pictures decoded are those expected, each with the
/// time it is due at in pictures since the start of its clip
fn assert_pictures(decoded: &[(usize, f64)], expected: &[(usize, f64)]) {
    let matching =
        decoded.len() == expected.len() && decoded.iter().zip(expected).all(|(&a, &b)| same(a, b));
    assert!(matching, "{decoded:?} is not {expected:?}");
}

#[test]
fn probe_reads_how_long_the_picture_lasts_and_whether_the_file_has_sound() {
    let (_directory, path) = movie(15, 5);
    assert_eq!(probe(&path), Ok((0.5, false)));
}

#[test]
fn file_that_is_missing_or_holds_no_h264_video_cannot_play() {
    let failed = |path: &Path| probe(path).expect_err("the file cannot play");
    assert_eq!(failed(Path::new("/nowhere/movie.mp4")), "file not found");
    let (directory, _) = movie(1, 1);
    let notes = directory.path().join("notes.mp4");
    fs::write(&notes, "not a video").expect("the file can be written");
    assert_eq!(failed(&notes), "holds no H.264 video");
}

/// Returns `bytes` with the four bytes after the box type `kind` and the
/// version and flags of the box, and the four after those, put in place by
/// `first` and `second`
fn patched(bytes: &[u8], kind: &[u8; 4], first: u32, second: u32) -> Vec<u8> {
    let at = bytes
        .windows(4)
        .position(|window| window == kind)
        .expect("the box is there")
        + 8;
    [
        &bytes[..at],
        &first.to_be_bytes(),
        &second.to_be_bytes(),
        &bytes[at + 8..],
    ]
    .concat()
}

#[test]
fn track_that_counts_more_samples_than_its_file_holds_bytes_cannot_play() {
    let (directory, path) = movie(5, 5);
    let bytes = fs::read(&path).expect("the video can be read");
    // Samples of one byte each, and more of them than the file holds
    // bytes, which take the time of one sample each to decode
    let sizes = patched(&bytes, b"stsz", 1, u32::MAX);
    let times = patched(&sizes, b"stts", 1, u32::MAX);
    let damaged = directory.path().join("damaged.mp4");
    fs::write(&damaged, times).expect("the video can be written");
    assert_eq!(
        probe(&damaged),
        Err("holds a damaged H.264 track".to_owned())
    );
}

#[test]
fn pictures_come_in_the_order_they_show_each_due_at_its_time() {
    let (_directory, path) = movie(15, 5);
    let expected: Vec<(usize, f64)> = (0..15).map(|number| (number, number as f64)).collect();
    assert_pictures(&decoded(&path, clip(0.0, 0.5, false), 0), &expected);
}

#[test]
fn clip_between_sync_samples_starts_with_the_picture_showing_at_its_start() {
    let (_directory, path) = movie(20, 8);
    // Picture 10 shows from a third of a second on, a little before the
    // start, and decodes only after pictures 8 and 9, which it refers to
    let start = 0.35 * f64::from(RATE);
    let expected: Vec<(usize, f64)> = (10..15)
        .map(|number| (number, (number as f64 - start).max(0.0)))
        .collect();
    assert_pictures(&decoded(&path, clip(0.35, 0.5, false), 0), &expected);
}

#[test]
fn later_pass_through_a_clip_that_loops_is_due_a_pass_later() {
    let (_directory, path) = movie(12, 4);
    // Pictures 3 to 5 make a tenth of a second, three pictures long
    let pass = |passed: f64| [3, 4, 5].map(|number| (number, passed + (number - 3) as f64));
    assert_pictures(&decoded(&path, clip(0.1, 0.2, true), 2), &pass(6.0));
}

#[test]
fn picture_that_fails_to_decode_leaves_the_next_group_its_own_times() {
    let (_directory, path) = broken_movie(10, 5, &[2]);
    let pictures = decoded(&path, clip(0.0, 10.0 / f64::from(RATE), false), 0);
    let later: Vec<(usize, f64)> = pictures
        .into_iter()
        .filter(|&(number, _)| number >= 5)
        .collect();
    let expected: Vec<(usize, f64)> = (5..10).map(|number| (number, number as f64)).collect();
    assert_pictures(&later, &expected);
}

#[test]
fn pass_that_decodes_no_picture_says_that_its_picture_cannot_be_decoded() {
    let (_directory, path) = broken_movie(6, 3, &[0, 1, 2, 3, 4, 5]);
    let sent = sent(&path, clip(0.0, 0.2, true), 0);
    assert!(
        matches!(sent.as_slice(), [Sent::Failed(reason)] if reason == UNDECODABLE),
        "{} sent",
        sent.len()
    );
}

#[test]
fn pass_stops_decoding_once_its_player_has_gone() {
    let (_directory, path) = movie(25, 25);
    let player = Arc::new(());
    let pictures = decoder::start(path, clip(0.0, 0.8, false), 0, Arc::downgrade(&player));
    drop(player);
    let sent: Vec<Sent> = pictures.iter().collect();
    assert!(sent.len() < 25, "{} sent", sent.len());
    assert!(sent.iter().all(|sent| matches!(sent, Sent::Picture(_))));
}
