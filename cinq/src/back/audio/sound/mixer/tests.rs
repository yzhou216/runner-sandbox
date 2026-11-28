// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of mixing sounds into the channels of a device, which need no
//! device: the mix renders into a buffer as the stream of a device would

mod devices;
mod fading;
mod routing;
mod samples;

use std::sync::Arc;

use kira::{
    AudioManager, AudioManagerSettings, Frame,
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use rtrb::{Consumer, Producer, RingBuffer};

use super::{
    config,
    mix::{Command, Gone, Mix},
    stalled, stream,
};
use crate::back::audio::{patch::Gains, sound::feed::Feed};

const RATE: u32 = 48000;

/// Frames of the ten milliseconds that a sound takes to fade out
const RAMP: usize = RATE as usize / 100;

/// Frames of each render, as a device asks for them
const FRAMES: usize = 256;

/// Left and right of the sound that the tests play, apart so that the
/// tests can tell where each went
const SOUND: Frame = Frame {
    left: 0.5,
    right: -0.25,
};

struct Bench {
    mix: Mix,
    rate: u32,
    channels: usize,
    commands: Producer<Command>,
    gone: Consumer<Gone>,
    /// Managers of the sounds, which hand the sounds their commands
    _managers: Vec<AudioManager<Feed>>,
}

impl Bench {
    fn new(channels: usize) -> Self {
        Self::at(RATE, channels)
    }

    fn at(rate: u32, channels: usize) -> Self {
        let (commands, taken) = RingBuffer::new(16);
        let (handed, gone) = RingBuffer::new(16);
        Self {
            mix: Mix::new(channels, rate, 8, taken, handed),
            rate,
            channels,
            commands,
            gone,
            _managers: Vec::new(),
        }
    }

    /// Plays a second of [`SOUND`] at `gains`
    fn play(mut self, id: u64, gains: Gains) -> Self {
        let rate = self.rate;
        let Ok(mut manager) = AudioManager::<Feed>::new(AudioManagerSettings {
            backend_settings: rate,
            ..AudioManagerSettings::default()
        });
        manager
            .play(StaticSoundData {
                sample_rate: rate,
                frames: Arc::from(vec![SOUND; rate as usize]),
                settings: StaticSoundSettings::default(),
                slice: None,
            })
            .expect("a sound plays");
        let renderer = manager
            .backend_mut()
            .take()
            .expect("kira starts the backend");
        let sent = self.commands.push(Command::add(id, renderer, gains));
        assert!(sent.is_ok(), "the ring takes the sound");
        self._managers.push(manager);
        self
    }

    fn remove(mut self, id: u64) -> Self {
        let sent = self.commands.push(Command::Remove(id));
        assert!(sent.is_ok(), "the ring takes the command");
        self
    }

    /// Renders [`FRAMES`] frames and returns each channel of them
    fn render(&mut self) -> Vec<Vec<f32>> {
        let mut out = vec![1.0; FRAMES * self.channels];
        self.mix.render(&mut out);
        (0..self.channels)
            .map(|channel| {
                out.iter()
                    .skip(channel)
                    .step_by(self.channels)
                    .copied()
                    .collect()
            })
            .collect()
    }
}

/// Gains of a device of `channels` channels, which send left and right
/// as `taps` lists them for each channel, and nothing to the others
fn gains(channels: usize, taps: &[(usize, [f32; 2])]) -> Gains {
    (0..channels)
        .map(|channel| {
            taps.iter()
                .find(|(tap, _)| *tap == channel)
                .map_or([0.0; 2], |(_, gain)| *gain)
        })
        .collect()
}

/// Returns whether every sample of `channel` past the first few, which
/// kira's interpolation leans into from before the start, is `value`
fn steady(channel: &[f32], value: f32) -> bool {
    channel[8..]
        .iter()
        .all(|sample| (sample - value).abs() < 1e-4)
}
