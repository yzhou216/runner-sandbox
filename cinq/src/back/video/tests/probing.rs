// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of probing video files for what they hold and how long they play

use std::path::Path;

use mp4_atom::{Avc1, Avcc, Codec, Stbl, Stsd};

use super::{
    boxes::revised,
    movie::{movie, movie_bytes, saved, sized_sequence_parameter_set},
    probe, tabled,
};

#[test]
fn probe_reads_how_long_the_picture_lasts_and_whether_the_file_has_sound() {
    let (_directory, path) = movie(15, 5);
    assert_eq!(probe(&path), Ok((0.5, false)));
}

#[test]
fn file_that_is_missing_or_holds_no_h264_video_cannot_play() {
    let failed = |path: &Path| probe(path).expect_err("the file cannot play");
    assert_eq!(failed(Path::new("/nowhere/movie.mp4")), "file not found");
    let (_directory, notes) = saved(b"not a video");
    assert_eq!(failed(&notes), "is cut short, or is not an MP4 file");
}

#[test]
fn file_cut_short_before_its_index_says_so_and_not_that_it_holds_no_video() {
    // Index lies at the end of the file, after the media data
    let bytes = movie_bytes(15, 5, &[]);
    let failed = |kept: usize| {
        let (_directory, path) = saved(&bytes[..kept]);
        probe(&path).expect_err("the file cannot play")
    };
    let words = "is cut short, or is not an MP4 file";
    assert_eq!(failed(bytes.len() / 2), words);
    assert_eq!(failed(bytes.len() - 10), words);
}

#[test]
fn track_that_counts_more_samples_than_its_file_holds_bytes_cannot_play() {
    let bytes = movie_bytes(5, 5, &[]);
    // Samples of one byte each, more of them than the file holds bytes,
    // and as many for the times to decode them
    let sizes = tabled(&bytes, b"stsz", &[1, u32::MAX]);
    let times = tabled(&sizes, b"stts", &[1, u32::MAX, 20]);
    let (_directory, damaged) = saved(&times);
    assert_eq!(
        probe(&damaged),
        Err("holds a damaged H.264 track".to_owned())
    );
}

#[test]
fn video_whose_parameter_set_declares_a_picture_larger_than_any_level_allows_cannot_play() {
    // Pictures of 16384 pixels each way, as a damaged size makes, which the
    // decoder would make in memory before it read a sample
    let huge = revised(&movie_bytes(5, 5, &[]), &|stbl| {
        let codecs = stbl.stsd.codecs.into_iter().map(|codec| match codec {
            Codec::Avc1(avc) => Codec::Avc1(Avc1 {
                avcc: Avcc::new(
                    &sized_sequence_parameter_set((1024, 1024)),
                    &avc.avcc.picture_parameter_sets[0],
                )
                .expect("the parameter sets are long enough"),
                ..avc
            }),
            other => other,
        });
        Stbl {
            stsd: Stsd {
                codecs: codecs.collect(),
            },
            ..stbl
        }
    });
    let (_directory, path) = saved(&huge);
    assert_eq!(
        probe(&path),
        Err("holds H.264 video larger than any level of H.264 allows".to_owned())
    );
}
