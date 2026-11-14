// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Conversion of a sound from the sample rate of its file to that of its
//! device
//!
//! kira converts by cubic interpolation without a filter, which on a Dante
//! network of 48 kHz leaves tones only 26 dB below a tone of 10 kHz in a
//! file of 44.1 kHz.  A sound of another rate than its device therefore
//! renders at its own rate, and the stream converts it by windowed sinc
//! interpolation, as rubato does it, which keeps them 89 dB below it.  A
//! sound at the rate of its device renders as it is.  Building a converter
//! takes about a millisecond, which only such sounds spend as they start.

use kira::backend::Renderer;
use rubato::{
    Async, FixedAsync, Resampler, SincInterpolationParameters, SincInterpolationType,
    WindowFunction, audioadapter_buffers::direct::InterleavedSlice,
};

use super::mix::CHUNK;

/// Points of the sinc filter, whose length rubato suggests for music
const POINTS: usize = 128;

pub(in crate::back::audio) struct Converter {
    resampler: Async<f32>,
    /// Stereo frames at the rate of the file, which the renderer fills
    input: Box<[f32]>,
    /// Stereo frames at the rate of the device, of which those from `taken`
    /// to `made` wait to be mixed
    output: Box<[f32]>,
    taken: usize,
    made: usize,
}

impl Converter {
    /// Returns a converter of `from` frames a second to `to`, or none where
    /// the rates match or rubato cannot convert between them
    pub(in crate::back::audio) fn new(from: u32, to: u32) -> Option<Self> {
        let parameters = SincInterpolationParameters {
            sinc_len: POINTS,
            f_cutoff: None,
            interpolation: SincInterpolationType::Cubic,
            oversampling_factor: 128,
            window: WindowFunction::BlackmanHarris2,
        };
        let ratio = f64::from(to) / f64::from(from);
        let resampler = (from != to && from > 0 && to > 0)
            .then(|| Async::new_sinc(ratio, 1.0, &parameters, CHUNK, 2, FixedAsync::Output).ok())
            .flatten()?;
        let input = vec![0.0; resampler.input_frames_max() * 2].into_boxed_slice();
        let output = vec![0.0; resampler.output_frames_max() * 2].into_boxed_slice();
        Some(Self {
            resampler,
            input,
            output,
            taken: 0,
            made: 0,
        })
    }

    /// Fills `out`, interleaved stereo frames at the rate of the device,
    /// with the sound that `renderer` makes at the rate of its file.  A
    /// failure of rubato, which its buffers rule out, leaves silence.
    pub(super) fn fill(&mut self, renderer: &mut Renderer, out: &mut [f32]) {
        let mut filled = 0;
        while filled < out.len() {
            if self.taken == self.made && !self.convert(renderer) {
                out[filled..].fill(0.0);
                return;
            }
            let count = (self.made - self.taken).min(out.len() - filled);
            out[filled..filled + count]
                .copy_from_slice(&self.output[self.taken..self.taken + count]);
            self.taken += count;
            filled += count;
        }
    }

    /// Renders the frames that rubato needs next and converts them, and
    /// returns whether it could
    fn convert(&mut self, renderer: &mut Renderer) -> bool {
        let needed = self.resampler.input_frames_next();
        let room = self.output.len() / 2;
        let input = &mut self.input[..needed * 2];
        renderer.process(input, 2);
        let made = InterleavedSlice::new(&*input, 2, needed)
            .ok()
            .zip(InterleavedSlice::new_mut(&mut self.output, 2, room).ok())
            .and_then(|(from, mut to)| {
                self.resampler
                    .process_into_buffer(&from, &mut to, None)
                    .ok()
            });
        match made {
            Some((_, frames)) => {
                self.taken = 0;
                self.made = frames * 2;
                frames > 0
            }
            None => false,
        }
    }
}
