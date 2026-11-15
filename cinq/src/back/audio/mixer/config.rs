// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sample formats and the configuration that a device opens with

use kira::backend::cpal::cpal::{
    BufferSize, Device, ErrorKind, SampleFormat, StreamConfig, traits::DeviceTrait,
};

use super::Fault;

/// Samples that a device can take, those that kira renders first, then
/// integers from the widest, as ASIO drivers, the hw devices of ALSA, and
/// Inferno's Dante device take
pub(super) const FORMATS: [SampleFormat; 4] = [
    SampleFormat::F32,
    SampleFormat::I32,
    SampleFormat::I24,
    SampleFormat::I16,
];

/// Returns the configuration of the stream of `device`: its default sample
/// rate, and the fewest channels that hold `channels`, or else all that it
/// has, in the first of [`FORMATS`] that it takes, with that format.  A
/// device of fewer channels than an output needs still opens, so that the
/// outputs within its channels play, and only that output tells.
pub(super) fn config(
    device: &Device,
    channels: u16,
) -> Result<(StreamConfig, SampleFormat), Fault> {
    let id = device.id().map(|id| id.to_string()).unwrap_or_default();
    let rate = device
        .default_output_config()
        .map_err(|error| match error.kind() {
            ErrorKind::UnsupportedConfig => Fault::Unplayable(unplayable(&id)),
            _ => Fault::Refused(error.to_string()),
        })?
        .sample_rate();
    let offered: Vec<_> = device
        .supported_output_configs()
        .map_err(|error| Fault::Refused(error.to_string()))?
        .filter(|range| (range.min_sample_rate()..=range.max_sample_rate()).contains(&rate))
        .map(|range| (range.sample_format(), range.channels()))
        .collect();
    let (format, count) = chosen(&offered, channels).ok_or_else(|| {
        Fault::Unplayable(if offered.is_empty() {
            format!("it offers no stream at {rate} Hz")
        } else {
            unplayable(&id)
        })
    })?;
    let config = StreamConfig {
        channels: count,
        sample_rate: rate,
        buffer_size: BufferSize::Default,
    };
    Ok((config, format))
}

/// Returns why the device of `id` cannot play, as it takes no samples that
/// cinq can play, such as the packed 24-bit samples that some USB
/// interfaces alone take on ALSA, which cpal cannot write.  The hw device of
/// a card names its plughw device, which converts them, to choose instead.
pub(super) fn unplayable(id: &str) -> String {
    let plug = id
        .strip_prefix("alsa:hw:")
        .map(|card| format!("; choose alsa:plughw:{card} instead"))
        .unwrap_or_default();
    format!("it takes no samples that cinq can play{plug}")
}

/// Returns the format and the count of channels to open from the formats
/// and counts that a device `offered`: the first of [`FORMATS`] with the
/// fewest channels that hold `channels`, or else the most channels that any
/// of them offers, in the first that offers so many, or none where it offers
/// none of them
pub(super) fn chosen(
    offered: &[(SampleFormat, u16)],
    channels: u16,
) -> Option<(SampleFormat, u16)> {
    let counts = |format: SampleFormat| {
        offered
            .iter()
            .filter(move |(offer, _)| *offer == format)
            .map(|(_, count)| *count)
    };
    let holding = || {
        FORMATS.into_iter().find_map(|format| {
            counts(format)
                .filter(|count| *count >= channels)
                .min()
                .map(|count| (format, count))
        })
    };
    let most = || {
        let widest = FORMATS
            .into_iter()
            .filter_map(|format| counts(format).max())
            .max()?;
        FORMATS
            .into_iter()
            .find(|format| counts(*format).any(|count| count == widest))
            .map(|format| (format, widest))
    };
    holding().or_else(most)
}
