// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Levels of sound and of light, their bounds, and the reading of the text
//! that gives them

use std::ops::RangeInclusive;

/// Level in dB that counts as silence, which is the bottom of every level
pub const SILENCE: f32 = -60.0;

/// Loudest level in dB that the operator can give a cue
pub const LOUDEST: f32 = 12.0;

pub const LEVELS: RangeInclusive<f32> = SILENCE..=LOUDEST;

/// Returns the level in dB that `text` reads as, with its unit or without
/// it, as in `-6` or `-6 dB`, if it lies between [`SILENCE`] and
/// [`LOUDEST`].  `-inf`, which other show controllers take for no level at
/// all, reads as [`SILENCE`], the bottom of every level.
pub fn parse_level(text: &str) -> Option<f32> {
    let number = text.trim().trim_end_matches(['d', 'D', 'b', 'B']).trim();
    if number.eq_ignore_ascii_case("-inf") {
        return Some(SILENCE);
    }
    number
        .parse::<f32>()
        .ok()
        .filter(|level| LEVELS.contains(level))
}

/// Returns the level that `text` gives a send, as in [`parse_level`]: the
/// inner `None` for empty text or `off`, which takes the send away, or the
/// outer `None` for text that reads as no level
pub fn parse_send(text: &str) -> Option<Option<f32>> {
    match text.trim() {
        "" => Some(None),
        off if off.eq_ignore_ascii_case("off") => Some(None),
        typed => parse_level(typed).map(Some),
    }
}

/// Levels of a light, in percent
pub const PERCENT: RangeInclusive<f32> = 0.0..=100.0;

/// Returns the light level that `text` reads as, as in `80` or `80%`: the
/// inner `None` for empty text, which leaves an instrument out of a light
/// cue, or the outer `None` for text that reads as no level
pub fn parse_percent(text: &str) -> Option<Option<f32>> {
    match text.trim().trim_end_matches('%').trim() {
        "" => Some(None),
        number => number
            .parse::<f32>()
            .ok()
            .filter(|level| PERCENT.contains(level))
            .map(Some),
    }
}
