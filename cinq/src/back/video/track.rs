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
//! track, and in the samples as well where the track is one of `avc3`.  Only
//! the [`moov`] box is read, whose tables [`locate`] turns into the places
//! of the samples.

mod colors;
mod locate;
mod moov;
mod tables;
#[cfg(test)]
mod tests;
mod timing;

use std::{
    collections::BTreeSet,
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
};

use rust_h264::nal::{NalUnit, NalUnitType, parse_avcc, parse_sps_info};

use super::decoder::{Declared, Turn};
use crate::{back::failure::unreadable, warn};
use locate::{Place, places};
use tables::{parameters, timed};
use timing::Sample;

const NO_VIDEO: &str = "holds no H.264 video";

/// No index fits in a copy that stopped short, or in a file that is not MP4
const NO_INDEX: &str = "is cut short, or is not an MP4 file";

/// The H.264 track, or a track that may be it, cannot be read
const DAMAGED_INDEX: &str = "holds a damaged index";

/// The tables of the H.264 track cannot be right
const DAMAGED: &str = "holds a damaged H.264 track";

/// Profiles of H.264 that play: Baseline, Main, and High, which hold eight
/// bits for each color and share the colors among four pixels, as the
/// pictures that the decoder turns into RGBA do
const PROFILES: [u8; 3] = [66, 77, 100];

/// Bytes before each parameter set of a track that give its length
const PARAMETER_LENGTH: usize = 4;

pub(super) const TOO_LARGE: &str = "holds H.264 video larger than any level of H.264 allows";

/// Pixels of the largest picture that a level of H.264 allows, which level
/// 6.2 sets, as 139264 macroblocks of 16 by 16 fill
const LARGEST: u64 = 139_264 * 256;

pub(super) struct Track {
    file: BufReader<File>,
    /// Bytes that the file held when it opened
    length: u64,
    /// Samples in the order they decode
    pub(super) samples: Vec<Sample>,
    places: Vec<Place>,
    /// Bytes before each NAL unit of a sample that give its length
    length_size: usize,
    /// Sequence and picture parameter sets of the track, each after its
    /// length in [`PARAMETER_LENGTH`] bytes
    parameters: Vec<u8>,
    /// Seconds that the picture lasts
    duration: f64,
    sound: bool,
    pub(super) turn: Turn,
    pub(super) colors: Declared,
    skipped: BTreeSet<String>,
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
        (track.duration as f32, track.sound)
    })
}

impl Track {
    /// Opens the video file at `path` and reads the tables of the H.264
    /// track with the lowest track ID, or tells why it cannot be played
    pub(super) fn open(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(unreadable)?;
        let length = file.metadata().map_err(|error| error.to_string())?.len();
        let mut file = BufReader::new(file);
        let movie = moov::read(&mut file, length).ok_or(NO_INDEX)?;
        let Some((trak, avc)) = &movie.video else {
            return Err(if movie.lost { DAMAGED_INDEX } else { NO_VIDEO }.to_owned());
        };
        if !PROFILES.contains(&avc.avc_profile_indication) {
            return Err(
                "holds H.264 video of a profile other than Baseline, Main, or High".to_owned(),
            );
        }
        let parameters = parameters(avc);
        let sets = parse_avcc(&parameters, PARAMETER_LENGTH);
        if oversized(&sets) {
            return Err(TOO_LARGE.to_owned());
        }
        let stbl = &trak.mdia.minf.stbl;
        let places = places(stbl, length).ok_or(DAMAGED)?;
        let colors = colors::declared(trak, &sets);
        let (samples, duration) = timed(stbl, places.len(), trak.mdia.mdhd.timescale);
        if samples.is_empty() || duration <= 0.0 {
            return Err(NO_VIDEO.to_owned());
        }
        Ok(Self {
            length_size: usize::from(avc.length_size),
            parameters,
            sound: movie.sound,
            turn: Turn::of(&trak.tkhd.matrix),
            colors,
            skipped: movie.skipped,
            samples,
            places,
            duration,
            length,
            file,
        })
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

/// Returns whether a sequence parameter set among `units` declares a picture
/// larger than any level allows, as one with a damaged size does.  The
/// decoder makes the picture as large as the set says, up to 16384 pixels
/// each way, which takes gigabytes, before it reads a sample.
pub(super) fn oversized(units: &[NalUnit<'_>]) -> bool {
    units
        .iter()
        .filter(|unit| unit.nal_unit_type == NalUnitType::Sps)
        .filter_map(|unit| parse_sps_info(unit).ok())
        .any(|sps| u64::from(sps.coded_width) * u64::from(sps.coded_height) > LARGEST)
}
