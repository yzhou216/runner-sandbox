// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of video files whose boxes or tables are damaged, which fail to
//! load with a reason and never hang or end cinq

use std::{fs, path::Path, sync::mpsc, thread};

use super::{
    PATIENCE,
    boxes::{narrow, reworked, shifted},
    movie::{movie_bytes, saved},
    tabled,
};
use crate::back::video::track::Track;

/// What probing finds in a video whose H.264 track cannot be read
pub(super) const DAMAGED_INDEX: &str = "holds a damaged index";

/// Returns the bytes of a test video of five pictures
pub(super) fn bytes() -> Vec<u8> {
    movie_bytes(5, 5, &[])
}

/// Returns `bytes` with a udta box holding `udta` after the tables of the
/// movie
pub(super) fn with_udta(bytes: &[u8], udta: &[u8]) -> Vec<u8> {
    reworked(bytes, &[b"moov"], &|body| {
        [body, narrow(&(*b"udta", udta.to_vec()))].concat()
    })
}

/// Returns whether playing the video at `path`, which opens it and reads
/// each of its samples, ends within the patience of the tests without a
/// panic
fn ends(path: &Path) -> bool {
    let (sender, found) = mpsc::channel();
    let path = path.to_owned();
    thread::spawn(move || {
        let _ = Track::open(&path).and_then(|mut track| {
            (0..track.samples.len()).try_for_each(|index| track.sample(index).map(drop))
        });
        let _ = sender.send(());
    });
    found.recv_timeout(PATIENCE).is_ok()
}

#[test]
fn video_with_a_field_of_its_tables_damaged_or_cut_short_ends_in_time() {
    let bytes = bytes();
    let (_directory, path) = saved(&[]);
    let moov = bytes
        .windows(4)
        .position(|window| window == b"moov")
        .expect("the box is there");
    // The four bytes at each place in the tables made zero, one, large, and
    // larger, as sizes and counts go wrong, and the file cut short there
    let damaged = |at: usize, value: u32| {
        let after = (at + 4).min(bytes.len());
        [&bytes[..at], &value.to_be_bytes(), &bytes[after..]].concat()
    };
    let hung: Vec<_> = (moov..bytes.len())
        .flat_map(|at| {
            [0, 1, i32::MAX as u32, u32::MAX]
                .map(|value| damaged(at, value))
                .into_iter()
                .chain([bytes[..at].to_vec()])
        })
        .enumerate()
        .filter(|(_, case)| {
            fs::write(&path, case).expect("the video can be written");
            !ends(&path)
        })
        .map(|(number, _)| number)
        .collect();
    assert_eq!(hung, Vec::<usize>::new());
}

#[test]
fn samples_that_lie_past_the_end_of_the_file_or_claim_more_than_it_holds_fail_to_read() {
    let past = reworked(&bytes(), &[b"moov"], &|body| shifted(&body, 1 << 20));
    // Sizes of the five samples, the first of them four gigabytes
    let huge = tabled(&bytes(), b"stsz", &[0, 5, u32::MAX, 1, 1, 1, 1]);
    [past, huge].into_iter().for_each(|bytes| {
        let (_directory, path) = saved(&bytes);
        let mut track = Track::open(&path).expect("the tables are right");
        assert_eq!(
            track.sample(0),
            Err("holds a damaged H.264 track".to_owned())
        );
    });
}
