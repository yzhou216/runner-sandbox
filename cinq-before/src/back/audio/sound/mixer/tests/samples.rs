// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the samples that the mix renders, past full scale and as integers

use kira::backend::cpal::cpal::I24;

use super::{Bench, FRAMES, SOUND, gains};

#[test]
fn sum_past_full_scale_clips_instead_of_wrapping_and_stops_at_the_last_value_of_24_bits() {
    let mut bench = Bench::new(2).play(1, gains(2, &[(0, [3.0, 0.0])]));
    let mut wide = vec![0.0_f32; FRAMES * 2];
    bench.mix.render(&mut wide);
    assert!(
        wide[16..]
            .chunks(2)
            .all(|frame| frame[0] > 0.999 && frame[0] < 1.0)
    );
    let mut narrow = vec![0_i16; FRAMES * 2];
    bench.mix.render(&mut narrow);
    assert!(narrow[16..].chunks(2).all(|frame| frame[0] == i16::MAX));
    let mut deep = vec![I24::new_unchecked(0); FRAMES * 2];
    bench.mix.render(&mut deep);
    assert!(
        deep[16..]
            .chunks(2)
            .all(|frame| frame[0].inner() == 8_388_607)
    );
}

#[test]
fn integer_samples_take_the_sound_at_its_level() {
    let mut bench = Bench::new(2).play(1, gains(2, &[(0, [1.0, 0.0]), (1, [0.0, 1.0])]));
    let mut wide = vec![0_i32; FRAMES * 2];
    bench.mix.render(&mut wide);
    let mut narrow = vec![0_i16; FRAMES * 2];
    bench.mix.render(&mut narrow);
    let near = |sample: f64, value: f32, full: f64| (sample / full - f64::from(value)).abs() < 1e-3;
    assert!(wide[16..].chunks(2).all(|frame| {
        near(f64::from(frame[0]), SOUND.left, 2_f64.powi(31))
            && near(f64::from(frame[1]), SOUND.right, 2_f64.powi(31))
    }));
    assert!(narrow.chunks(2).all(|frame| {
        near(f64::from(frame[0]), SOUND.left, 2_f64.powi(15))
            && near(f64::from(frame[1]), SOUND.right, 2_f64.powi(15))
    }));
}
