// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Stream of a device, and what its thread tells the mixer

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Instant,
};

use kira::backend::cpal::cpal::{
    self, Device, ErrorKind, FromSample, SizedSample, Stream, StreamConfig, traits::DeviceTrait,
};

use super::mix::Mix;

/// What the thread of a stream tells the mixer: its latest error, whether
/// it must be built anew, and when it last asked for sound
pub(super) struct Tells {
    pub(super) error: Arc<Mutex<Option<String>>>,
    pub(super) lost: Arc<AtomicBool>,
    pub(super) started: Instant,
    pub(super) called: Arc<AtomicU64>,
}

/// Opens the stream of `device` in samples of `T`, which `mix` fills
pub(super) fn stream<T>(
    device: &Device,
    config: StreamConfig,
    mut mix: Mix,
    tells: Tells,
) -> Result<Stream, String>
where
    T: SizedSample + FromSample<f32>,
{
    let Tells {
        error,
        lost,
        started,
        called,
    } = tells;
    device
        .build_output_stream(
            config,
            move |out: &mut [T], _: &cpal::OutputCallbackInfo| {
                mix.render(out);
                let since = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
                called.store(since, Ordering::Relaxed);
            },
            move |reported: cpal::Error| {
                if let Ok(mut latest) = error.lock() {
                    *latest = Some(reported.to_string());
                }
                if ends(reported.kind()) {
                    lost.store(true, Ordering::Relaxed);
                }
            },
            None,
        )
        .map_err(|error| error.to_string())
}

/// Returns whether an error of `kind` means that the stream must be built
/// anew: that its device or host has gone, or that it no longer plays as
/// it was built to, as after a change of the sample rate of the device
pub(super) fn ends(kind: ErrorKind) -> bool {
    matches!(
        kind,
        ErrorKind::DeviceNotAvailable | ErrorKind::HostUnavailable | ErrorKind::StreamInvalidated
    )
}
