// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Mixer of one audio device, which plays the sounds that play on it into
//! the channels of the device that their outputs name
//!
//! The device opens with the fewest channels that it offers at its default
//! sample rate that hold the channels asked for, in the first samples of
//! [`config::FORMATS`] that it takes.  Opening with the most it offers would
//! ask 64 channels of the plug devices of ALSA, which take any number.  A
//! device with fewer channels than asked for opens with all that it has, so
//! that a typo in the channels of one output silences that output alone.
//! The mix itself runs on the thread of the stream, as [`mix`] tells.

#[cfg(test)]
mod bench;
mod config;
mod mix;
mod stall;
mod stream;
#[cfg(test)]
mod tests;

use std::{collections::HashMap, iter, sync::atomic::Ordering, time::Duration};

use kira::backend::{
    Renderer,
    cpal::cpal::{
        self, Device, I24, SampleFormat, Stream,
        traits::{HostTrait, StreamTrait},
    },
};
use rtrb::{Consumer, Producer, RingBuffer};

use crate::back::audio::{
    Fault,
    devices::device,
    patch::{Gains, Key},
    sound::opener::Hold,
};
use config::config;
pub(in crate::back::audio) use mix::Mix;
use mix::{Command, Gone};
use stall::stalled;
use stream::{Tells, stream};

/// Commands that can wait for the stream at once, far more than one
/// update of the workspace sends
const COMMANDS: usize = 4096;

/// Sounds that one device plays at once before its list of them grows on
/// the thread of the stream
const SOUNDS: usize = 1024;

/// Returns the device whose ID is `device`, or the only one named `name` on
/// a machine that gives it another ID, or the default device without an ID
fn chosen_device(device: Option<&str>, name: Option<&str>) -> Result<Device, String> {
    match device {
        Some(id) => device::find(id, name),
        None => cpal::default_host()
            .default_output_device()
            .ok_or_else(|| "there is no default audio device".to_owned()),
    }
}

/// Mixers of the devices, by their keys, or why each could not open
pub(in crate::back::audio) type Mixers = HashMap<Key, Result<Mixer, Fault>>;

/// Returns the channels that the device of `key` opened with, or none where
/// it is not open
pub(in crate::back::audio) fn channels_of(mixers: &Mixers, key: &Key) -> Option<u16> {
    mixers.get(key)?.as_ref().ok().map(Mixer::channels)
}

pub(in crate::back::audio) struct Mixer {
    /// Hold on the stream of the device, which plays as long as the mixer
    /// is kept, and which is absent only in a test that plays through a mix
    /// that it renders
    _hold: Option<Hold>,
    commands: Producer<Command>,
    gone: Consumer<Gone>,
    sample_rate: u32,
    channels: u16,
    /// Channels that the outputs needed as the device opened, which can be
    /// more than it has
    asked: u16,
    tells: Tells,
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
    ) -> Result<(Self, Stream), Fault> {
        Self::start(
            &chosen_device(device, name).map_err(Fault::Missing)?,
            channels,
        )
    }

    /// Returns how many channels the device whose ID is `device`, or the
    /// default device without one, would open with for `channels`, without
    /// opening it, or none where it is not there or cannot play
    pub(super) fn offers(device: Option<&str>, name: Option<&str>, channels: u16) -> Option<u16> {
        let (config, _) = config(&chosen_device(device, name).ok()?, channels).ok()?;
        Some(config.channels)
    }

    fn start(found: &Device, channels: u16) -> Result<(Self, Stream), Fault> {
        let (config, format) = config(found, channels)?;
        let (commands, taken) = RingBuffer::new(COMMANDS);
        let (handed, gone) = RingBuffer::new(COMMANDS);
        let mix = Mix::new(
            usize::from(config.channels),
            config.sample_rate,
            SOUNDS,
            taken,
            handed,
        );
        let tells = Tells::new();
        let stream = match format {
            SampleFormat::I32 => stream::<i32>(found, config, mix, tells.clone()),
            SampleFormat::I24 => stream::<I24>(found, config, mix, tells.clone()),
            SampleFormat::I16 => stream::<i16>(found, config, mix, tells.clone()),
            _ => stream::<f32>(found, config, mix, tells.clone()),
        }
        .map_err(Fault::Refused)?;
        stream
            .play()
            .map_err(|error| Fault::Refused(error.to_string()))?;
        let mixer = Self {
            _hold: None,
            commands,
            gone,
            sample_rate: config.sample_rate,
            channels: config.channels,
            asked: channels,
            tells,
            next: 0,
        };
        Ok((mixer, stream))
    }

    /// Returns the mixer with `hold` on the stream that plays it, which
    /// closes the stream once the mixer goes
    pub(super) fn held(self, hold: Hold) -> Self {
        Self {
            _hold: Some(hold),
            ..self
        }
    }

    pub(super) fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub(in crate::back::audio) fn channels(&self) -> u16 {
        self.channels
    }

    pub(in crate::back::audio) fn asked(&self) -> u16 {
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
    pub(in crate::back::audio) fn collect(&mut self) {
        iter::from_fn(|| self.gone.pop().ok()).for_each(drop);
    }

    /// Returns why the stream stopped, which is the latest error that it
    /// told, if any.  The stream has stopped if it told an error after which
    /// it must be built anew, or if it has asked for no sound for
    /// [`stall::STALL`].  Errors of the first kind tell at once, since an
    /// ASIO driver that asks to be reset may play on, and a device that goes
    /// or takes another sample rate need not hold the cues.  Other errors
    /// alone do not tell, since ALSA plays on after an underrun or a failed
    /// look at its buffer, where WASAPI ends the stream after any failed
    /// call.
    pub(in crate::back::audio) fn failure(&self) -> Option<String> {
        // Stream that has called has done so after its start, so zero
        // nanoseconds mean that it has not
        let tells = &self.tells;
        let called = Some(tells.called.load(Ordering::Relaxed))
            .filter(|nanos| *nanos > 0)
            .map(Duration::from_nanos);
        let stalled = stalled(tells.started.elapsed(), called);
        (tells.lost.load(Ordering::Relaxed) || stalled).then(|| {
            tells
                .error
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
