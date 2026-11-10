// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! MP4 file that holds the samples of a test video, whose boxes mp4-atom
//! writes
//!
//! The media data comes first and the tables after it, as an encoder that
//! cannot know them before it has written every sample leaves them.

use std::{fs, path::Path};

use itertools::Itertools;
use mp4_atom::{
    Avc1, Avcc, Codec, Dinf, Dref, DrefEntry, Encode, FourCC, Ftyp, Hdlr, Mdat, Mdhd, Mdia, Minf,
    Moov, Mvhd, Stbl, Stco, Stsc, StscEntry, Stsd, Stss, Stsz, StszSamples, Stts, SttsEntry, Tkhd,
    Trak, Url, Visual, Vmhd,
};

use super::{
    RATE, SIZE, TIMESCALE,
    stream::{picture_parameter_set, sequence_parameter_set},
};

/// Samples that each chunk of the track holds, but the last, which holds
/// those left
const PER_CHUNK: usize = 4;

/// Units of time a second in the whole movie
const MOVIE_TIMESCALE: u32 = 1000;

/// Writes the MP4 file at `path` whose one track of H.264 holds `samples`,
/// each after the length of its NAL unit, and that decoding can start at
/// every `sync_every` of them
pub(super) fn write(path: &Path, samples: &[Vec<u8>], sync_every: usize) {
    let count = u32::try_from(samples.len()).expect("test videos are short");
    let delta = TIMESCALE / RATE;
    let ftyp = encoded(&Ftyp {
        major_brand: b"isom".into(),
        minor_version: 512,
        compatible_brands: [b"isom", b"iso2", b"avc1", b"mp41"]
            .map(FourCC::from)
            .to_vec(),
    });
    let mdat = encoded(&Mdat {
        data: samples.concat(),
    });
    // The first sample follows the header of the media data, which is
    // eight bytes
    let first = u32::try_from(ftyp.len() + 8).expect("test videos are short");
    let chunks = samples.chunks(PER_CHUNK).collect::<Vec<_>>();
    let offsets = chunks
        .iter()
        .scan(first, |at, chunk| {
            let offset = *at;
            *at += chunk.iter().map(|sample| sample.len() as u32).sum::<u32>();
            Some(offset)
        })
        .collect();
    let runs = chunks
        .iter()
        .zip(1..)
        .map(|(chunk, first_chunk)| StscEntry {
            first_chunk,
            samples_per_chunk: chunk.len() as u32,
            sample_description_index: 1,
        })
        .dedup_by(|next, run| next.samples_per_chunk == run.samples_per_chunk)
        .collect();
    let side = |pixels: usize| u16::try_from(pixels).expect("test videos are small");
    let (width, height) = (side(SIZE.0), side(SIZE.1));
    let avcc = Avcc::new(&sequence_parameter_set(), &picture_parameter_set())
        .expect("the parameter sets are long enough");
    let stbl = Stbl {
        stsd: Stsd {
            codecs: vec![Codec::Avc1(Avc1 {
                visual: Visual {
                    width,
                    height,
                    ..Visual::default()
                },
                avcc,
                ..Avc1::default()
            })],
        },
        stts: Stts {
            entries: vec![SttsEntry {
                sample_count: count,
                sample_delta: delta,
            }],
        },
        stss: Some(Stss {
            entries: (1..=count).step_by(sync_every).collect(),
        }),
        stsc: Stsc { entries: runs },
        stsz: Stsz {
            samples: StszSamples::Different {
                sizes: samples.iter().map(|sample| sample.len() as u32).collect(),
            },
        },
        stco: Some(Stco { entries: offsets }),
        ..Stbl::default()
    };
    let duration = u64::from(count * delta);
    let moov = encoded(&Moov {
        mvhd: Mvhd {
            timescale: MOVIE_TIMESCALE,
            duration: duration * u64::from(MOVIE_TIMESCALE) / u64::from(TIMESCALE),
            next_track_id: 2,
            ..Mvhd::default()
        },
        trak: vec![Trak {
            tkhd: Tkhd {
                track_id: 1,
                enabled: true,
                in_movie: true,
                width: width.into(),
                height: height.into(),
                ..Tkhd::default()
            },
            mdia: Mdia {
                mdhd: Mdhd {
                    timescale: TIMESCALE,
                    duration,
                    language: "und".into(),
                    ..Mdhd::default()
                },
                hdlr: Hdlr {
                    handler: b"vide".into(),
                    name: String::new(),
                },
                minf: Minf {
                    vmhd: Some(Vmhd::default()),
                    dinf: Dinf {
                        dref: Dref {
                            entries: vec![DrefEntry::Url(Url::default())],
                        },
                    },
                    stbl,
                    ..Minf::default()
                },
            },
            ..Trak::default()
        }],
        ..Moov::default()
    });
    fs::write(path, [ftyp, mdat, moov].concat()).expect("the video file can be written");
}

/// Returns the bytes of `atom` with its header
fn encoded(atom: &impl Encode) -> Vec<u8> {
    let mut bytes = Vec::new();
    atom.encode(&mut bytes).expect("the box can be encoded");
    bytes
}
