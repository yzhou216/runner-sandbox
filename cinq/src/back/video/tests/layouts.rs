// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests that video files whose boxes lie in the other ways that the
//! format allows play as they do in the way that the tests write them

use std::{cell::RefCell, iter, path::Path};

use mp4_atom::{
    Avc3, Avcc, Codec, Ctts, CttsEntry, Stbl, Stco, Stsc, StscEntry, Stsd, Stsz, StszSamples,
};

use super::{
    assert_pictures,
    boxes::{Part, ilst_udta, narrow, revised, shifted, split, wide},
    clip, decoded, from_at,
    movie::{movie_bytes, saved},
};
use crate::back::video::track::{Track, probe};

/// Returns the bytes of each sample of the video at `path`, or why it
/// cannot play
fn played(path: &Path) -> Result<Vec<Vec<u8>>, String> {
    let mut track = Track::open(path)?;
    (0..track.samples.len())
        .map(|index| track.sample(index))
        .collect()
}

/// Returns the boxes of a test video, `ftyp`, `mdat`, and `moov` in that
/// order, along with the samples that it plays
fn parts() -> ([Part; 3], Vec<Vec<u8>>) {
    let bytes = movie_bytes(5, 5, &[]);
    let parts = split(&bytes).try_into().expect("the boxes are three");
    (parts, written(&bytes).expect("the video plays"))
}

/// Writes `bytes` as a video and returns the samples that it plays
fn written(bytes: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    let (_directory, path) = saved(bytes);
    played(&path)
}

#[test]
fn files_whose_boxes_lie_in_the_other_ways_that_the_format_allows_play() {
    let ([ftyp, mdat, (moov, body)], samples) = parts();
    let with_wide_header = |kind: [u8; 4]| {
        split(&body)
            .iter()
            .flat_map(|part| {
                if part.0 == kind {
                    wide(part)
                } else {
                    narrow(part)
                }
            })
            .collect::<Vec<u8>>()
    };
    // A media data box with a header of sixteen bytes, which moves the
    // samples that the moov box locates
    let wide_mdat = [
        narrow(&ftyp),
        wide(&mdat),
        narrow(&(moov, shifted(&body, 8))),
    ];
    // Moov box and its trak box with such headers
    let wide_moov = [
        narrow(&ftyp),
        narrow(&mdat),
        wide(&(moov, with_wide_header(*b"trak"))),
    ];
    // Media data box that claims no bytes, after the moov box
    let (_, data) = &mdat;
    let to_the_end = [
        narrow(&ftyp),
        narrow(&(moov, shifted(&body, body.len() + 8))),
        [&[0; 4][..], b"mdat", data].concat(),
    ];
    // Udta and meta boxes that ffmpeg writes
    let tagged = (
        moov,
        [body.clone(), narrow(&(*b"udta", ilst_udta()))].concat(),
    );
    let with_tags = [narrow(&ftyp), narrow(&mdat), narrow(&tagged)];
    // No ftyp box, a box that the reader skips taking its place so that the
    // samples stay where the moov box locates them
    let no_ftyp = [
        narrow(&(*b"free", ftyp.1)),
        narrow(&mdat),
        narrow(&(moov, body)),
    ];
    [wide_mdat, wide_moov, to_the_end, with_tags, no_ftyp]
        .iter()
        .enumerate()
        .for_each(|(layout, boxes)| {
            assert_eq!(written(&boxes.concat()), Ok(samples.clone()), "{layout}")
        });
}

/// Returns each sample of the video that `bytes` make, with the units of
/// time at which it shows, of which the test video holds 600 a second, and
/// whether decoding can start at it
fn timed(bytes: &[u8]) -> Vec<(i64, bool)> {
    let (_directory, path) = saved(bytes);
    let track = Track::open(&path).expect("the video plays");
    track
        .samples
        .iter()
        .map(|sample| ((sample.shows * 600.0).round() as i64, sample.sync))
        .collect()
}

#[test]
fn ctts_of_version_one_shows_samples_by_its_negative_offsets() {
    let entry = |(sample_count, sample_offset)| CttsEntry {
        sample_count,
        sample_offset,
    };
    let ctts = Ctts {
        entries: [(1, -40), (1, 20), (1, -20), (2, 0)].map(entry).to_vec(),
    };
    let bytes = revised(&movie_bytes(5, 5, &[]), &|stbl| Stbl {
        ctts: Some(ctts.clone()),
        ..stbl
    });
    // The samples decode every 20 units, so that they show 40 units before
    // and 20 after, and 20 before, and then as they decode, which counts
    // from the first picture to show
    let shown: Vec<i64> = timed(&bytes).into_iter().map(|(shows, _)| shows).collect();
    assert_eq!(shown, [0, 80, 60, 100, 120]);
}

#[test]
fn track_without_a_list_of_sync_samples_starts_decoding_at_every_sample() {
    let listed = timed(&movie_bytes(12, 4, &[]));
    assert_eq!(listed.iter().filter(|(_, sync)| *sync).count(), 3);
    let unlisted = timed(&revised(&movie_bytes(12, 4, &[]), &|stbl| Stbl {
        stss: None,
        ..stbl
    }));
    assert!(unlisted.len() == 12 && unlisted.iter().all(|(_, sync)| *sync));
}

#[test]
fn avc3_track_whose_first_sample_holds_the_parameter_sets_plays() {
    let bytes = movie_bytes(5, 5, &[]);
    let samples = written(&bytes).expect("the video plays");
    let sets = RefCell::new(Vec::new());
    let tabled = revised(&bytes, &|stbl| {
        if let Some(Codec::Avc1(avc)) = stbl.stsd.codecs.first() {
            let (sps, pps) = (
                &avc.avcc.sequence_parameter_sets,
                &avc.avcc.picture_parameter_sets,
            );
            *sets.borrow_mut() = sps.iter().chain(pps).cloned().collect();
        }
        stbl
    });
    // Each set after its length, as a sample holds a NAL unit
    let in_band = sets
        .into_inner()
        .iter()
        .flat_map(|set| [(set.len() as u32).to_be_bytes().to_vec(), set.clone()].concat())
        .collect::<Vec<u8>>();
    let [ftyp, mdat, (moov, body)] = split(&tabled).try_into().expect("the boxes are three");
    // The samples lie in a box of their own before the moov box, the first
    // after the parameter sets, which the avcC box leaves out
    let kept = [in_band.clone(), samples[0].clone()].concat();
    let moved = [kept.clone(), samples[1..].concat()].concat();
    let at = (narrow(&ftyp).len() + narrow(&mdat).len() + 8) as u32;
    let file = [
        narrow(&ftyp),
        narrow(&mdat),
        narrow(&(*b"free", moved)),
        narrow(&(moov, body)),
    ]
    .concat();
    let with_avc3 = revised(&file, &|stbl| {
        let codecs = stbl.stsd.codecs.iter().map(|codec| match codec {
            Codec::Avc1(avc) => Codec::Avc3(Avc3 {
                visual: avc.visual.clone(),
                avcc: Avcc {
                    sequence_parameter_sets: Vec::new(),
                    picture_parameter_sets: Vec::new(),
                    ..avc.avcc.clone()
                },
                ..Avc3::default()
            }),
            other => other.clone(),
        });
        let sizes = iter::once(kept.len())
            .chain(samples[1..].iter().map(Vec::len))
            .map(|size| size as u32)
            .collect();
        Stbl {
            stsd: Stsd {
                codecs: codecs.collect(),
            },
            stsz: Stsz {
                samples: StszSamples::Different { sizes },
            },
            stco: Some(Stco { entries: vec![at] }),
            stsc: Stsc {
                entries: vec![StscEntry {
                    first_chunk: 1,
                    samples_per_chunk: 5,
                    sample_description_index: 1,
                }],
            },
            ..stbl
        }
    });
    let (_directory, path) = saved(&with_avc3);
    assert_eq!(probe(&path), Ok((5.0 / 30.0, false)));
    assert_pictures(
        &decoded(&path, clip(0.0, 5.0 / 30.0, false), 0),
        &from_at(0..=4, 0.0),
    );
}
