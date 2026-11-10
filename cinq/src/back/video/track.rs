// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! H.264 track of an MP4 file: its samples, the times at which they show,
//! and their bytes as the decoder reads them
//!
//! MP4 holds the samples of a track in the order they decode, which with
//! B-frames differs from the order they show in.  The tables of the track
//! give each sample the time at which it decodes, and the offset from there
//! to the time at which it shows, which [`timing`] reads.  The picture
//! starts with the first sample to show, so that it starts with the sound,
//! as the edit lists that encoders write for B-frames shift it.  Each
//! sample holds NAL units after their lengths, as the decoder reads them,
//! and the parameter sets that decoding needs are in the description of the
//! track rather than in its samples.  The file is read by the tracks of its
//! [`moov`] box alone, whose tables [`locate`] turns into the places of the
//! samples.

mod locate;
mod moov;
#[cfg(test)]
mod tests;
mod timing;

use std::{
    collections::BTreeSet,
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    iter,
    path::Path,
};

use rust_h264::nal::{NalUnit, parse_avcc};

use crate::{back::load::unreadable, warn};
use locate::{Place, places};
use timing::{Sample, timeline};

const NO_VIDEO: &str = "holds no H.264 video";

/// Why a file holds no picture to play when its H.264 track, or a track that
/// may be it, cannot be read
const DAMAGED_INDEX: &str = "holds a damaged index";

/// Why a file holds no picture to play when the tables of its H.264 track
/// cannot be right
const DAMAGED: &str = "holds a damaged H.264 track";

/// Profiles of H.264 that play: Baseline, Main, and High, which hold eight
/// bits for each color and share the colors among four pixels, as the
/// pictures that the decoder turns into RGBA do
const PROFILES: [u8; 3] = [66, 77, 100];

/// Bytes before each parameter set of a track that give its length
const PARAMETER_LENGTH: usize = 4;

pub(super) struct Track {
    file: BufReader<File>,
    /// Bytes that the file held when it opened
    length: u64,
    /// Samples in the order they decode
    pub(super) samples: Vec<Sample>,
    /// Where each sample lies in the file
    places: Vec<Place>,
    /// Bytes before each NAL unit of a sample that give its length
    length_size: usize,
    /// Sequence and picture parameter sets of the track, each after its
    /// length in [`PARAMETER_LENGTH`] bytes
    parameters: Vec<u8>,
    /// Seconds that the picture lasts
    duration: f64,
    sound: bool,
    /// What the file left out because it cannot be read, as the words that
    /// say so
    pub(super) skipped: BTreeSet<String>,
}

/// Reads how many seconds the picture of a video file plays and whether
/// the file holds a sound track besides, or why the picture cannot be
/// played.  This reads the tables of the tracks of the file without
/// decoding a picture, and warns of what it skips for being unreadable.
pub(in crate::back) fn probe(path: &Path) -> Result<(f32, bool), String> {
    Track::open(path).map(|track| {
        track
            .skipped
            .iter()
            .for_each(|skipped| warn::once(&format!("{}: {skipped}", path.display())));
        track.summary()
    })
}

impl Track {
    /// Opens the video file at `path` and reads the tables of its first
    /// H.264 track, or tells why it cannot be played
    pub(super) fn open(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(unreadable)?;
        let length = file.metadata().map_err(|error| error.to_string())?.len();
        let mut file = BufReader::new(file);
        let movie = moov::read(&mut file, length).ok_or(NO_VIDEO)?;
        let Some((trak, avc)) = &movie.video else {
            return Err(if movie.lost { DAMAGED_INDEX } else { NO_VIDEO }.to_owned());
        };
        if !PROFILES.contains(&avc.avc_profile_indication) {
            return Err("holds H.264 video in a profile that cannot be decoded".to_owned());
        }
        let stbl = &trak.mdia.minf.stbl;
        let places = places(stbl, length).ok_or(DAMAGED)?;
        // Tables of the track, whose runs of equal values give each sample
        // the time it takes to decode and its offset from there to the time
        // it shows, and list the samples that decoding can start at
        let deltas = stbl
            .stts
            .entries
            .iter()
            .flat_map(|entry| iter::repeat_n(entry.sample_delta, entry.sample_count as usize))
            .take(places.len());
        // Both versions of the box hold 32 bits, which mp4-atom widens and
        // the timeline reads signed
        let offsets = stbl
            .ctts
            .iter()
            .flat_map(|ctts| &ctts.entries)
            .flat_map(|entry| {
                iter::repeat_n(entry.sample_offset as i32, entry.sample_count as usize)
            });
        let syncs = stbl.stss.as_ref().map(|stss| stss.entries.as_slice());
        let (samples, duration) = timeline(deltas, offsets, syncs, trak.mdia.mdhd.timescale);
        if samples.is_empty() || duration <= 0.0 {
            return Err(NO_VIDEO.to_owned());
        }
        let parameters = avc
            .sequence_parameter_sets
            .iter()
            .chain(&avc.picture_parameter_sets)
            .flat_map(|set| {
                // avcC counts the bytes of each set in two bytes, so that
                // the length always fits
                let length = set.len() as u32;
                length.to_be_bytes().into_iter().chain(set.iter().copied())
            })
            .collect();
        Ok(Self {
            length_size: usize::from(avc.length_size),
            parameters,
            sound: movie.sound,
            skipped: movie.skipped,
            samples,
            places,
            duration,
            length,
            file,
        })
    }

    /// Returns how many seconds the picture lasts and whether the file holds
    /// a sound track, which [`probe`] returns
    pub(super) fn summary(&self) -> (f32, bool) {
        (self.duration as f32, self.sound)
    }

    pub(super) fn parameter_sets(&self) -> Vec<NalUnit<'_>> {
        parse_avcc(&self.parameters, PARAMETER_LENGTH)
    }

    /// Reads the bytes of the sample at `index` in the order of decoding,
    /// which fails when the tables of the track put the sample past the end
    /// of the file, as a file cut short does
    pub(super) fn sample(&mut self, index: usize) -> Result<Vec<u8>, String> {
        let Place { offset, size } = *self.places.get(index).ok_or(DAMAGED)?;
        // Both come from the file, so that the bytes must lie in it before a
        // buffer for them is made
        let inside = offset
            .checked_add(u64::from(size))
            .is_some_and(|end| end <= self.length);
        if !inside {
            return Err(DAMAGED.to_owned());
        }
        let mut bytes = vec![0; size as usize];
        self.file
            .seek(SeekFrom::Start(offset))
            .and_then(|_| self.file.read_exact(&mut bytes))
            .map_err(|error| error.to_string())?;
        Ok(bytes)
    }

    /// Returns the NAL units of the bytes of a sample, each of which follows
    /// its length.  A sample cut short ends at its last whole unit.
    pub(super) fn units<'a>(&self, sample: &'a [u8]) -> Vec<NalUnit<'a>> {
        parse_avcc(sample, self.length_size)
    }

    pub(super) fn span(&self, start: f64, end: f64) -> Option<(usize, usize, f64)> {
        timing::span(&self.samples, start, end)
    }
}
