// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Mixer of one audio device, which plays the sounds that play on it into
//! the channels of the device that their outputs name
//!
//! The device opens with the fewest channels that it offers at its default
//! sample rate that hold the channels asked for, in the first samples of
//! [`FORMATS`] that it takes.  Opening with the most it offers would ask 64
//! channels of the plug devices of ALSA, which take any number.  A device
//! with fewer channels than asked for opens with all that it has, so that
//! a typo in the channels of one output silences that output alone.  The
//! mix itself runs on the thread of the stream, as [`mix`] tells.

mod mix;
#[cfg(test)]
mod tests;

use std::{
    iter,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use kira::backend::{
    Renderer,
    cpal::cpal::{
        self, BufferSize, Device, ErrorKind, FromSample, I24, SampleFormat, SizedSample, Stream,
        StreamConfig,
        traits::{DeviceTrait, HostTrait, StreamTrait},
    },
};
use rtrb::{Consumer, Producer, RingBuffer};

use super::device;
pub(super) use mix::Gains;
use mix::{Command, Gone, Mix};

/// Commands that can wait for the stream at once, far more than one
/// update of the workspace sends
const COMMANDS: usize = 4096;

/// Sounds that one device plays at once before its list of them grows on
/// the thread of the stream
const SOUNDS: usize = 1024;

/// Time without a call for sound after which a stream counts as stopped,
/// longer than the 1.9 seconds that a new stream on the ALSA plugin of
/// PulseAudio waits after its first calls
const STALL: Duration = Duration::from_secs(3);

pub(super) struct Mixer {
    /// Stream of the device, which plays as long as the mixer is kept
    _stream: Stream,
    commands: Producer<Command>,
    gone: Consumer<Gone>,
    sample_rate: u32,
    channels: u16,
    /// Channels that the outputs needed as the device opened, which can be
    /// more than it has
    asked: u16,
    /// Latest error of the stream, which it tells from its own thread
    error: Arc<Mutex<Option<String>>>,
    /// Whether the stream has told an error after which it must be built
    /// anew
    lost: Arc<AtomicBool>,
    /// Moment the stream started, from which `called` counts
    started: Instant,
    /// Nanoseconds after `started` of the latest call of the stream for
    /// sound
    called: Arc<AtomicU64>,
    /// Identity that the next sound takes
    next: u64,
}

impl Mixer {
    /// Opens the device whose ID is `device`, or the only one named `name`
    /// on a machine that gives it another ID, or the default device without
    /// an ID, with `channels` channels or more
    pub(super) fn open(
        device: Option<&str>,
        name: Option<&str>,
        channels: u16,
    ) -> Result<Self, String> {
        let found = match device {
            Some(id) => device::find(id, name)?,
            None => cpal::default_host()
                .default_output_device()
                .ok_or("there is no default audio device")?,
        };
        Self::start(&found, channels).map_err(|error| match device {
            Some(id) => format!("cannot play on {id}: {error}"),
            None => error,
        })
    }

    fn start(found: &Device, channels: u16) -> Result<Self, String> {
        let (config, format) = config(found, channels)?;
        let (commands, taken) = RingBuffer::new(COMMANDS);
        let (handed, gone) = RingBuffer::new(COMMANDS);
        let mix = Mix::new(usize::from(config.channels), SOUNDS, taken, handed);
        let error = Arc::new(Mutex::new(None));
        let lost = Arc::new(AtomicBool::new(false));
        let started = Instant::now();
        let called = Arc::new(AtomicU64::new(0));
        let tells = Tells {
            error: Arc::clone(&error),
            lost: Arc::clone(&lost),
            started,
            called: Arc::clone(&called),
        };
        let stream = match format {
            SampleFormat::I32 => stream::<i32>(found, config, mix, tells),
            SampleFormat::I24 => stream::<I24>(found, config, mix, tells),
            SampleFormat::I16 => stream::<i16>(found, config, mix, tells),
            _ => stream::<f32>(found, config, mix, tells),
        }?;
        stream.play().map_err(|error| error.to_string())?;
        Ok(Self {
            _stream: stream,
            commands,
            gone,
            sample_rate: config.sample_rate,
            channels: config.channels,
            asked: channels,
            error,
            lost,
            started,
            called,
            next: 0,
        })
    }

    pub(super) fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub(super) fn channels(&self) -> u16 {
        self.channels
    }

    pub(super) fn asked(&self) -> u16 {
        self.asked
    }

    /// Mixes the sound that `renderer` makes at `gains`, and returns the
    /// identity of the sound in the mixer
    pub(super) fn add(&mut self, renderer: Renderer, gains: Gains) -> Result<u64, String> {
        let id = self.next;
        self.send(Command::add(id, renderer, gains))?;
        self.next += 1;
        Ok(id)
    }

    /// Ramps the sound of `id` to `gains`
    pub(super) fn route(&mut self, id: u64, gains: Gains) {
        let _ = self.send(Command::Route(id, gains));
    }

    /// Fades out the sound of `id`, which then goes
    pub(super) fn remove(&mut self, id: u64) {
        let _ = self.send(Command::Remove(id));
    }

    /// Frees what the stream no longer needs
    pub(super) fn collect(&mut self) {
        iter::from_fn(|| self.gone.pop().ok()).for_each(drop);
    }

    /// Returns why the stream stopped, if it has told an error after which
    /// it must be built anew, or has asked for no sound for [`STALL`]: the
    /// latest error that it told, if any.  Errors of the first kind tell at
    /// once, since an ASIO driver that asks to be reset may play on, and a
    /// device that goes or takes another sample rate need not hold the
    /// cues.  Other errors alone do not tell, since ALSA plays on after an
    /// underrun or a failed look at its buffer, where WASAPI ends the
    /// stream after any failed call.
    pub(super) fn failure(&self) -> Option<String> {
        let called = Duration::from_nanos(self.called.load(Ordering::Relaxed));
        let stalled = self.started.elapsed().saturating_sub(called) >= STALL;
        (self.lost.load(Ordering::Relaxed) || stalled).then(|| {
            self.error
                .lock()
                .ok()
                .and_then(|error| error.clone())
                .unwrap_or_else(|| "the device stopped asking for sound".to_owned())
        })
    }

    fn send(&mut self, command: Command) -> Result<(), String> {
        self.commands
            .push(command)
            .map_err(|_| "the mixer of the device has more to do than it can take".to_owned())
    }
}

/// Samples that a device can take, those that kira renders first, then
/// integers from the widest, as ASIO drivers, the hw devices of ALSA, and
/// Inferno's Dante device take
const FORMATS: [SampleFormat; 4] = [
    SampleFormat::F32,
    SampleFormat::I32,
    SampleFormat::I24,
    SampleFormat::I16,
];

/// What the thread of a stream tells the mixer: its latest error, whether
/// it must be built anew, and when it last asked for sound
struct Tells {
    error: Arc<Mutex<Option<String>>>,
    lost: Arc<AtomicBool>,
    started: Instant,
    called: Arc<AtomicU64>,
}

/// Returns the configuration of the stream of `device`: its default sample
/// rate, and the fewest channels that hold `channels`, or else all that it
/// has, in the first of [`FORMATS`] that it takes, with that format.  A
/// device of fewer channels than an output needs still opens, so that the
/// outputs within its channels play, and only that output tells.
fn config(device: &Device, channels: u16) -> Result<(StreamConfig, SampleFormat), String> {
    let rate = device
        .default_output_config()
        .map_err(|error| error.to_string())?
        .sample_rate();
    let offered: Vec<_> = device
        .supported_output_configs()
        .map_err(|error| error.to_string())?
        .filter(|range| (range.min_sample_rate()..=range.max_sample_rate()).contains(&rate))
        .map(|range| (range.sample_format(), range.channels()))
        .collect();
    let (format, count) = chosen(&offered, channels).ok_or_else(|| {
        if offered.is_empty() {
            format!("it offers no stream at {rate} Hz")
        } else {
            format!("it takes no samples that cinq can play at {rate} Hz")
        }
    })?;
    let config = StreamConfig {
        channels: count,
        sample_rate: rate,
        buffer_size: BufferSize::Default,
    };
    Ok((config, format))
}

/// Returns the format and the count of channels to open from the formats
/// and counts that a device `offered`: the first of [`FORMATS`] with the
/// fewest channels that hold `channels`, or else the first with the most
/// channels, or none where it offers none of them
fn chosen(offered: &[(SampleFormat, u16)], channels: u16) -> Option<(SampleFormat, u16)> {
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
        FORMATS
            .into_iter()
            .find_map(|format| counts(format).max().map(|count| (format, count)))
    };
    holding().or_else(most)
}

/// Opens the stream of `device` in samples of `T`, which `mix` fills
fn stream<T>(
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
fn ends(kind: ErrorKind) -> bool {
    matches!(
        kind,
        ErrorKind::DeviceNotAvailable | ErrorKind::HostUnavailable | ErrorKind::StreamInvalidated
    )
}
