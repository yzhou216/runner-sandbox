// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Mixer of one audio device, which plays the sounds that play on it into
//! the channels of the device that their outputs name
//!
//! The device opens with the fewest channels that it offers at its default
//! sample rate that hold the channels asked for, in the first samples of
//! [`FORMATS`] that it takes.  Opening with the most it offers would ask 64
//! channels of the plug devices of ALSA, which take any number.  The mix
//! itself runs on the thread of the stream, as [`mix`] tells.

mod mix;
#[cfg(test)]
mod tests;

use std::{
    iter,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use kira::backend::{
    Renderer,
    cpal::cpal::{
        self, BufferSize, Device, FromSample, I24, SampleFormat, SizedSample, Stream, StreamConfig,
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
    /// Latest error of the stream, which it tells from its own thread
    error: Arc<Mutex<Option<String>>>,
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
        let started = Instant::now();
        let called = Arc::new(AtomicU64::new(0));
        let tells = Tells {
            error: Arc::clone(&error),
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
            error,
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

    /// Returns why the stream stopped, if it has asked for no sound for
    /// [`STALL`]: the latest error that it told, if any.  Its errors alone
    /// do not tell, since ALSA plays on after an underrun or a failed look
    /// at its buffer, where WASAPI ends the stream after any failed call.
    pub(super) fn failure(&self) -> Option<String> {
        let called = Duration::from_nanos(self.called.load(Ordering::Relaxed));
        (self.started.elapsed().saturating_sub(called) >= STALL).then(|| {
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

/// What the thread of a stream tells the mixer: its latest error, and when
/// it last asked for sound
struct Tells {
    error: Arc<Mutex<Option<String>>>,
    started: Instant,
    called: Arc<AtomicU64>,
}

/// Returns the configuration of the stream of `device`: its default sample
/// rate, and the fewest channels that hold `channels` in the first of
/// [`FORMATS`] that it takes, with that format
fn config(device: &Device, channels: u16) -> Result<(StreamConfig, SampleFormat), String> {
    let rate = device
        .default_output_config()
        .map_err(|error| error.to_string())?
        .sample_rate();
    let ranges: Vec<_> = device
        .supported_output_configs()
        .map_err(|error| error.to_string())?
        .filter(|range| {
            (range.min_sample_rate()..=range.max_sample_rate()).contains(&rate)
                && range.channels() >= channels
        })
        .collect();
    let (format, fewest) = FORMATS
        .into_iter()
        .find_map(|format| {
            let fewest = ranges
                .iter()
                .filter(|range| range.sample_format() == format)
                .map(|range| range.channels())
                .min()?;
            Some((format, fewest))
        })
        .ok_or_else(|| {
            format!("it takes no samples that cinq can play on {channels} channels at {rate} Hz")
        })?;
    let config = StreamConfig {
        channels: fewest,
        sample_rate: rate,
        buffer_size: BufferSize::Default,
    };
    Ok((config, format))
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
            },
            None,
        )
        .map_err(|error| error.to_string())
}
