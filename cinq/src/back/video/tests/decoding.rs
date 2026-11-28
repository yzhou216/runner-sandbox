// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of decoding the pictures of a clip, each due at its time

use std::sync::Arc;

use mp4_atom::{Ctts, CttsEntry, Matrix, Stbl, Tkhd, Trak};

use super::{
    assert_pictures,
    boxes::{retracked, revised},
    clip, decoded,
    decoder::{self, Sent, UNDECODABLE},
    from_at,
    movie::{RATE, broken_movie, movie, movie_bytes, saved},
    sent,
};

/// One whole turn in the fixed-point numbers of a matrix
const ONE: i32 = 1 << 16;

#[test]
fn pictures_come_in_the_order_they_show_each_due_at_its_time() {
    let (_directory, path) = movie(15, 5);
    assert_pictures(
        &decoded(&path, clip(0.0, 0.5, false), 0),
        &from_at(0..=14, 0.0),
    );
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
    assert_pictures(
        &decoded(&path, clip(0.1, 0.2, true), 2),
        &from_at(3..=5, 6.0),
    );
}

#[test]
fn picture_that_fails_to_decode_leaves_the_next_group_its_own_times() {
    let (_directory, path) = broken_movie(10, 5, &[2]);
    let pictures = decoded(&path, clip(0.0, 10.0 / f64::from(RATE), false), 0);
    let later: Vec<(usize, f64)> = pictures
        .into_iter()
        .filter(|&(number, _)| number >= 5)
        .collect();
    assert_pictures(&later, &from_at(5..=9, 5.0));
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
    let pass = clip(0.0, 0.8, false).pass(0).expect("the clip has a pass");
    let pictures = decoder::start(path, pass, Arc::downgrade(&player));
    drop(player);
    let sent: Vec<Sent> = pictures.iter().collect();
    assert!(sent.len() < 25, "{} sent", sent.len());
    assert!(sent.iter().all(|sent| matches!(sent, Sent::Picture(_))));
}

#[test]
fn pictures_of_a_track_whose_matrix_turns_it_come_upright() {
    let matrix = |a, b, c, d| Matrix {
        a,
        b,
        c,
        d,
        ..Matrix::default()
    };
    let turns = [
        (matrix(0, ONE, -ONE, 0), (48, 64)),
        (matrix(-ONE, 0, 0, -ONE), (64, 48)),
        (matrix(0, -ONE, ONE, 0), (48, 64)),
    ];
    turns.into_iter().for_each(|(matrix, size)| {
        let bytes = retracked(&movie_bytes(3, 3, &[]), &|trak| Trak {
            tkhd: Tkhd {
                matrix: matrix.clone(),
                ..trak.tkhd
            },
            ..trak
        });
        let (_directory, path) = saved(&bytes);
        let sizes: Vec<(u32, u32)> = sent(&path, clip(0.0, 0.1, false), 0)
            .iter()
            .filter_map(|sent| match sent {
                Sent::Picture(frame) => Some((frame.picture.width, frame.picture.height)),
                _ => None,
            })
            .collect();
        assert_eq!(sizes, [size; 3]);
    });
}

#[test]
fn track_that_holds_back_more_pictures_than_h264_does_fails_to_play() {
    // The last sample shows before all the others, so that none can go out
    // before it decodes
    let entry = |sample_count, sample_offset| CttsEntry {
        sample_count,
        sample_offset,
    };
    let ctts = Ctts {
        entries: vec![entry(39, 0), entry(1, -100_000)],
    };
    let bytes = revised(&movie_bytes(40, 40, &[]), &|stbl| Stbl {
        ctts: Some(ctts.clone()),
        ..stbl
    });
    let (_directory, path) = saved(&bytes);
    let sent = sent(&path, clip(0.0, 1.0, false), 0);
    assert!(
        matches!(sent.as_slice(), [Sent::Failed(reason)] if reason == UNDECODABLE),
        "{} sent",
        sent.len()
    );
}
