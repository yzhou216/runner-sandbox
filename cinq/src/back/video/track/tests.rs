// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the times at which the samples of a track show, B-frames among
//! them, and of where decoding a part of the picture starts, in open groups
//! of pictures as well

use super::{
    Sample, Track,
    timing::{span, timeline},
};
use crate::back::video::tests::movie::movie;

/// Returns samples in the order of decoding, each of which shows at its
/// number of pictures, a second each, and those of `syncs` can start
/// decoding
fn samples(shows: &[u32], syncs: &[usize]) -> Vec<Sample> {
    shows
        .iter()
        .enumerate()
        .map(|(index, &shows)| Sample {
            shows: f64::from(shows),
            sync: syncs.contains(&index),
        })
        .collect()
}

/// Returns a closed group of I, P, and two B-frames, then an open group
/// whose two B-frames decode after its I-frame and show before it
fn open_groups() -> Vec<Sample> {
    samples(&[0, 3, 1, 2, 6, 4, 5, 9, 7, 8], &[0, 4])
}

#[test]
fn decoding_starts_at_a_sync_sample_that_shows_no_later_than_the_first_picture() {
    let groups = open_groups();
    // Picture 4 decodes after the I-frame of its open group, which shows
    // later, so decoding starts at the group before it
    assert_eq!(span(&groups, 4.0, 5.5), Some((0, 6, 4.0)));
    assert_eq!(span(&groups, 6.0, 7.5), Some((4, 8, 6.0)));
}

#[test]
fn samples_show_at_their_offsets_from_the_first_picture() {
    // I, P, and two B-frames, which decode in that order and show as I, B,
    // B, and P, two pictures of 512 units after they start to decode
    let (samples, end) = timeline(
        [512; 4].into_iter(),
        [1024, 2048, 512, 512].into_iter(),
        Some(&[1]),
        15360,
    );
    let picture = 512.0 / 15360.0;
    let expected =
        [(0.0, true), (3.0, false), (1.0, false), (2.0, false)].map(|(shows, sync)| Sample {
            shows: shows * picture,
            sync,
        });
    assert_eq!(samples, expected);
    assert_eq!(end, 4.0 * picture);
}

#[test]
fn samples_without_offsets_or_a_list_of_sync_samples_show_as_they_decode() {
    let (samples, end) = timeline([20; 3].into_iter(), [].into_iter(), None, 600);
    let shows: Vec<f64> = samples.iter().map(|sample| sample.shows * 30.0).collect();
    assert_eq!(shows, [0.0, 1.0, 2.0]);
    assert!(samples.iter().all(|sample| sample.sync));
    assert_eq!(end, 0.1);
}

#[test]
fn decoding_starts_at_the_sync_sample_before_the_picture_that_shows_first() {
    let (_directory, path) = movie(20, 8);
    let track = Track::open(&path).expect("the video opens");
    let syncs: Vec<usize> = track
        .samples
        .iter()
        .enumerate()
        .filter(|(_, sample)| sample.sync)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(syncs, [0, 8, 16]);
    let (from, to, first) = track.span(0.35, 0.5).expect("the part has pictures");
    assert_eq!((from, to), (8, 14));
    assert!((first * 30.0 - 10.0).abs() < 1e-9, "{first}");
}
