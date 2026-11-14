// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Mixing on the thread of the stream of a device
//!
//! The thread must never wait, so it takes no lock and frees no memory.
//! The main thread sends it sounds and their gains over one ring, and it
//! hands back over another what it no longer needs, for the main thread to
//! free.  A change of gains ramps over [`RAMP`] frames, and a sound that
//! goes fades out over as many, so that neither clicks.

use kira::backend::{Renderer, cpal::cpal::FromSample};

use super::convert::Converter;
use rtrb::{Consumer, Producer};

/// Gain from the left and the right of a sound into each channel of a
/// device, in order
pub(in crate::back::audio) type Gains = Box<[[f32; 2]]>;

/// Frames that a sound renders at a time, which the scratch buffer holds
pub(super) const CHUNK: usize = 512;

/// Frames over which gains change, ten milliseconds at 48 kHz, as long as
/// the default fade of kira
pub(super) const RAMP: usize = 480;

/// Sound that the stream mixes, by the identity that the main thread gave
/// it
pub(super) struct Slot {
    id: u64,
    renderer: Renderer,
    /// Conversion of the sound from the rate of its file, where it differs
    /// from that of the device
    converter: Option<Converter>,
    /// Gains as they stand, which ramp toward `target`
    gains: Gains,
    target: Gains,
    /// Frames left of the ramp
    ramp: usize,
    /// Whether the sound fades out and goes once its ramp ends
    leaving: bool,
}

pub(super) enum Command {
    Add(Box<Slot>),
    Route(u64, Gains),
    Remove(u64),
}

impl Command {
    pub(super) fn add(
        id: u64,
        renderer: Renderer,
        converter: Option<Converter>,
        gains: Gains,
    ) -> Self {
        Self::Add(Box::new(Slot {
            id,
            renderer,
            converter,
            gains: gains.clone(),
            target: gains,
            ramp: 0,
            leaving: false,
        }))
    }
}

/// What the stream no longer needs, which the main thread frees
pub(super) enum Gone {
    Slot(#[allow(dead_code)] Box<Slot>),
    Gains(#[allow(dead_code)] Gains),
}

pub(super) struct Mix {
    channels: usize,
    /// Sounds in the boxes that the main thread made, which stay boxed so
    /// that this thread neither frees nor moves them
    #[allow(clippy::vec_box)]
    slots: Vec<Box<Slot>>,
    /// Stereo frames of one chunk of a sound
    scratch: Box<[f32]>,
    /// Frames of one chunk of the mix, in floating point whatever samples
    /// the device takes
    mixed: Box<[f32]>,
    commands: Consumer<Command>,
    gone: Producer<Gone>,
}

impl Mix {
    /// `capacity` sounds fit before the list of sounds grows, which would
    /// take memory on the thread of the stream
    pub(super) fn new(
        channels: usize,
        capacity: usize,
        commands: Consumer<Command>,
        gone: Producer<Gone>,
    ) -> Self {
        Self {
            channels,
            slots: Vec::with_capacity(capacity),
            scratch: vec![0.0; CHUNK * 2].into_boxed_slice(),
            mixed: vec![0.0; CHUNK * channels].into_boxed_slice(),
            commands,
            gone,
        }
    }

    /// Fills `out`, frames of interleaved channels in the samples that the
    /// device takes, with every sound, which kira renders in floating point
    pub(super) fn render<T: FromSample<f32>>(&mut self, out: &mut [T]) {
        self.obey();
        let Self {
            channels,
            slots,
            scratch,
            mixed,
            ..
        } = self;
        let channels = *channels;
        slots
            .iter_mut()
            .for_each(|slot| slot.renderer.on_start_processing());
        out.chunks_mut(CHUNK * channels).for_each(|chunk| {
            let mixed = &mut mixed[..chunk.len()];
            mixed.fill(0.0);
            slots.iter_mut().for_each(|slot| {
                let stereo = &mut scratch[..chunk.len() / channels * 2];
                match &mut slot.converter {
                    Some(converter) => converter.fill(&mut slot.renderer, stereo),
                    None => slot.renderer.process(stereo, 2),
                }
                slot.mix_into(mixed, channels, stereo);
            });
            chunk
                .iter_mut()
                .zip(mixed.iter())
                .for_each(|(sample, value)| *sample = T::from_sample_(*value));
        });
        self.part();
    }

    fn obey(&mut self) {
        while let Ok(command) = self.commands.pop() {
            match command {
                Command::Add(slot) => self.slots.push(slot),
                Command::Route(id, gains) => {
                    let old = match self.slot(id) {
                        Some(slot) if !slot.leaving => {
                            slot.ramp = RAMP;
                            std::mem::replace(&mut slot.target, gains)
                        }
                        _ => gains,
                    };
                    let _ = self.gone.push(Gone::Gains(old));
                }
                Command::Remove(id) => {
                    if let Some(slot) = self.slot(id) {
                        slot.target.fill([0.0; 2]);
                        slot.ramp = RAMP;
                        slot.leaving = true;
                    }
                }
            }
        }
    }

    fn slot(&mut self, id: u64) -> Option<&mut Slot> {
        self.slots
            .iter_mut()
            .find(|slot| slot.id == id)
            .map(|slot| &mut **slot)
    }

    /// Hands back the sounds that have faded out.  A full ring drops them
    /// here instead, which no ring of this size comes to.
    fn part(&mut self) {
        while let Some(index) = self
            .slots
            .iter()
            .position(|slot| slot.leaving && slot.ramp == 0)
        {
            let _ = self.gone.push(Gone::Slot(self.slots.swap_remove(index)));
        }
    }
}

impl Slot {
    /// Adds the stereo frames of `stereo` into `out`, frames of `channels`
    /// channels, at the gains of the slot
    fn mix_into(&mut self, out: &mut [f32], channels: usize, stereo: &[f32]) {
        let left = self.ramp;
        let span = left.min(stereo.len() / 2);
        self.gains
            .iter_mut()
            .zip(self.target.iter())
            .enumerate()
            .filter(|(_, (from, to))| **from != [0.0; 2] || **to != [0.0; 2])
            .for_each(|(channel, (from, to))| {
                let start = *from;
                let step = if left == 0 {
                    [0.0; 2]
                } else {
                    [
                        (to[0] - start[0]) / left as f32,
                        (to[1] - start[1]) / left as f32,
                    ]
                };
                let at = |frame: usize| {
                    if frame < span {
                        let done = (frame + 1) as f32;
                        [start[0] + step[0] * done, start[1] + step[1] * done]
                    } else {
                        *to
                    }
                };
                out.iter_mut()
                    .skip(channel)
                    .step_by(channels)
                    .zip(stereo.as_chunks::<2>().0)
                    .enumerate()
                    .for_each(|(frame, (sample, sound))| {
                        let gain = at(frame);
                        *sample += sound[0] * gain[0] + sound[1] * gain[1];
                    });
                *from = match span {
                    _ if span == left => *to,
                    0 => start,
                    _ => at(span - 1),
                };
            });
        self.ramp -= span;
    }
}
