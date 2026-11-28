// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the tables and the description of a track give the decoder: the times
//! of its samples and its parameter sets

use std::iter;

use mp4_atom::{Avcc, Stbl};

use super::timing::{Sample, timeline};

/// Returns the `count` samples of the track that `stbl` describes, with the
/// seconds that the picture lasts.  The tables hold runs of equal values for
/// the time each sample takes to decode and its offset from there to the
/// time it shows, and list the samples that decoding can start at.
pub(super) fn timed(stbl: &Stbl, count: usize, timescale: u32) -> (Vec<Sample>, f64) {
    let deltas = stbl
        .stts
        .entries
        .iter()
        .flat_map(|entry| iter::repeat_n(entry.sample_delta, entry.sample_count as usize))
        .take(count);
    // Both versions of the box hold 32 bits, which mp4-atom widens and
    // the timeline reads signed
    let offsets = stbl
        .ctts
        .iter()
        .flat_map(|ctts| &ctts.entries)
        .flat_map(|entry| iter::repeat_n(entry.sample_offset as i32, entry.sample_count as usize));
    let syncs = stbl.stss.as_ref().map(|stss| stss.entries.as_slice());
    timeline(deltas, offsets, syncs, timescale)
}

/// Returns the sequence and picture parameter sets of `avc`, each after its
/// length in [`super::PARAMETER_LENGTH`] bytes
pub(super) fn parameters(avc: &Avcc) -> Vec<u8> {
    avc.sequence_parameter_sets
        .iter()
        .chain(&avc.picture_parameter_sets)
        .flat_map(|set| {
            // avcC counts the bytes of each set in two bytes, so that
            // the length always fits
            let length = set.len() as u32;
            length.to_be_bytes().into_iter().chain(set.iter().copied())
        })
        .collect()
}
