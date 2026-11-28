// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the fade out of a sound that goes

use super::{Bench, FRAMES, RAMP, SOUND, gains};

/// Removes the sound of ID 1 from `bench`, which plays it on the first
/// channel, and returns the bench with that channel as the sound fades out
/// over `ramp` frames
fn fade_out(mut bench: Bench, ramp: usize) -> (Bench, Vec<f32>) {
    bench.render();
    let mut bench = bench.remove(1);
    let fading = (0..ramp.div_ceil(FRAMES))
        .flat_map(|_| bench.render().swap_remove(0))
        .collect();
    (bench, fading)
}

#[test]
fn sound_that_goes_fades_out_and_comes_back_to_be_freed() {
    let (mut bench, fading) = fade_out(Bench::new(2).play(1, gains(2, &[(0, [1.0, 0.0])])), RAMP);
    assert!(fading[..RAMP].windows(2).all(|pair| pair[1] <= pair[0]));
    assert!(fading[0] > 0.9 * SOUND.left && fading[RAMP - 1].abs() < 1e-6);
    assert!(bench.gone.pop().is_ok(), "the sound comes back to be freed");
    assert!(bench.render()[0].iter().all(|sample| *sample == 0.0));
}

#[test]
fn fade_out_lasts_ten_milliseconds_at_any_rate() {
    let rate: u32 = 96_000;
    let ramp = (rate / 100) as usize;
    let bench = Bench::at(rate, 2).play(1, gains(2, &[(0, [1.0, 0.0])]));
    let (_, fading) = fade_out(bench, ramp);
    assert!(fading[ramp - 2] > 1e-4 && fading[ramp - 1].abs() < 1e-6);
}
