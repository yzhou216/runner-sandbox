// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! When the samples of a track show, and which samples decoding a part of
//! the picture reads
//!
//! In a closed group of pictures, which most encoders make, every picture
//! of the group shows no sooner than the sync sample that starts it.  In an
//! open one, the pictures that decode after its sync sample can show before
//! it, and refer to the group before it, so that decoding them has to start
//! at an earlier sync sample.

use std::iter;

/// Sample of a track
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::back::video) struct Sample {
    /// Seconds into the picture at which the sample shows
    pub(in crate::back::video) shows: f64,
    /// Whether decoding can start at the sample
    pub(in crate::back::video) sync: bool,
}

/// Returns the samples of a track in the order of decoding, each with the
/// time at which it shows, counted in seconds from the first picture to
/// show, along with the time at which the picture ends.  Each sample takes
/// the time of its `deltas` to decode and shows its `offsets` after it
/// decodes, which are none past the end of the offsets, in units of which
/// `timescale` make a second.  Decoding can start at the samples whose
/// numbers, counted from one, `syncs` lists, or at every sample without the
/// list.
pub(super) fn timeline(
    deltas: impl Iterator<Item = u32>,
    offsets: impl Iterator<Item = i32>,
    syncs: Option<&[u32]>,
    timescale: u32,
) -> (Vec<Sample>, f64) {
    // Time at which each sample shows and then stops showing, in units of
    // the timescale
    let shown: Vec<(i64, i64)> = deltas
        .scan(0_i64, |decoded, delta| {
            let at = *decoded;
            *decoded += i64::from(delta);
            Some((at, i64::from(delta)))
        })
        .zip(offsets.chain(iter::repeat(0)))
        .map(|((decoded, delta), offset)| {
            let shows = decoded + i64::from(offset);
            (shows, shows + delta)
        })
        .collect();
    let first = shown.iter().map(|&(shows, _)| shows).min().unwrap_or(0);
    let end = shown.iter().map(|&(_, ends)| ends).max().unwrap_or(0);
    let seconds = |time: i64| (time - first) as f64 / f64::from(timescale.max(1));
    let sync = |number: u32| syncs.is_none_or(|syncs| syncs.binary_search(&number).is_ok());
    let samples = shown
        .iter()
        .zip(1..)
        .map(|(&(shows, _), number)| Sample {
            shows: seconds(shows),
            sync: sync(number),
        })
        .collect();
    (samples, seconds(end))
}

/// Returns the samples that decoding the picture from `start` to `end`
/// seconds reads, by their indices in the order of decoding: from the last
/// sync sample before the picture that shows at `start` that shows no later
/// than that picture, which decoding has to start at, to the last sample
/// that shows before `end`.  Along with them comes the time at which the
/// picture that shows at `start` shows, which is the first of the part.
pub(super) fn span(samples: &[Sample], start: f64, end: f64) -> Option<(usize, usize, f64)> {
    let by_time = |a: &(usize, &Sample), b: &(usize, &Sample)| a.1.shows.total_cmp(&b.1.shows);
    let indexed = || samples.iter().enumerate();
    let (first, shown) = indexed()
        .filter(|(_, sample)| sample.shows <= start)
        .max_by(by_time)
        .or_else(|| indexed().min_by(by_time))?;
    let from = samples[..=first]
        .iter()
        .rposition(|sample| sample.sync && sample.shows <= shown.shows)
        .unwrap_or(0);
    let to = samples
        .iter()
        .rposition(|sample| sample.shows < end)
        .unwrap_or(first);
    Some((from, to.max(first), shown.shows))
}
