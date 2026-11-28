// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the samples of a track lie in its file
//!
//! A track keeps its samples in chunks, which lie one after another in
//! the media data of the file.  One table gives the offset of each chunk, a
//! second how many samples each run of chunks holds, and a third the size
//! of each sample, so that a sample lies after those that share its chunk.
//! The tables come from the file, so that nothing here takes a count or an
//! offset of theirs on trust: a count of samples of one size that its file
//! cannot hold, a size that no sample of H.264 has, and runs of chunks out
//! of order, give no samples, and runs and chunks that the tables leave out
//! hold none.

#[cfg(test)]
mod tests;

use std::iter;

use mp4_atom::{Stbl, StszSamples};

/// Bytes of the least sample of H.264, a NAL unit of one byte after its
/// length in four.  A table of samples of one size that has none could give
/// a sample for each byte of the file, or for no bytes at all, which costs
/// memory many times the size of the file.
const LEAST: u32 = 5;

/// Bytes of the file that hold a sample, which may reach past its end
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Place {
    pub(super) offset: u64,
    pub(super) size: u32,
}

/// Returns the places of the samples of the track whose tables are `stbl`,
/// in the order of decoding, or none when the tables cannot be right for a
/// file of `length` bytes
pub(super) fn places(stbl: &Stbl, length: u64) -> Option<Vec<Place>> {
    // Each of runs out of order could walk every chunk, which would take
    // hours when the tables are long
    let in_order = stbl.stsc.entries.is_sorted_by_key(|run| run.first_chunk);
    match &stbl.stsz.samples {
        StszSamples::Identical { count, size } => {
            let fits = u64::from(*count) * u64::from(*size) <= length && *size >= LEAST;
            (in_order && fits).then(|| chunked(stbl, iter::repeat_n(*size, *count as usize)))
        }
        StszSamples::Different { sizes } => in_order.then(|| chunked(stbl, sizes.iter().copied())),
    }
}

/// Places the samples of `sizes` in the chunks of `stbl`, which end at the
/// last sample that both the chunks and the sizes hold
fn chunked(stbl: &Stbl, sizes: impl Iterator<Item = u32>) -> Vec<Place> {
    let offsets: Vec<u64> = match (&stbl.stco, &stbl.co64) {
        (Some(stco), _) => stco.entries.iter().copied().map(u64::from).collect(),
        (None, Some(co64)) => co64.entries.clone(),
        (None, None) => Vec::new(),
    };
    let runs = &stbl.stsc.entries;
    // Index of a chunk, which the file numbers from one, kept within the
    // chunks that it lists
    let chunk = |number: u32| (number.saturating_sub(1) as usize).min(offsets.len());
    // Chunk of each sample, and whether the sample is the first of it.  Each
    // run lasts until the next one starts, and the last one to the last chunk.
    let chunks = runs.iter().enumerate().flat_map(|(index, run)| {
        let end = runs
            .get(index + 1)
            .map_or(offsets.len(), |next| chunk(next.first_chunk));
        let per_chunk = run.samples_per_chunk;
        (chunk(run.first_chunk)..end)
            .flat_map(move |chunk| (0..per_chunk).map(move |sample| (chunk, sample == 0)))
    });
    chunks
        .zip(sizes)
        .scan(0_u64, |at, ((chunk, first), size)| {
            *at = if first { *offsets.get(chunk)? } else { *at };
            let place = Place { offset: *at, size };
            *at = at.saturating_add(u64::from(size));
            Some(place)
        })
        .collect()
}
