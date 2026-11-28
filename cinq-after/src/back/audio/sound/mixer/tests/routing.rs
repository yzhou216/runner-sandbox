// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the channels that a sound plays on

use super::{Bench, SOUND, gains, steady};

#[test]
fn sound_plays_on_the_channels_that_its_gains_name_and_nowhere_else() {
    let mut bench = Bench::new(4).play(1, gains(4, &[(2, [1.0, 0.0]), (3, [0.0, 1.0])]));
    let channels = bench.render();
    assert!(
        channels[0]
            .iter()
            .chain(&channels[1])
            .all(|sample| *sample == 0.0)
    );
    assert!(steady(&channels[2], SOUND.left), "{:?}", &channels[2][..12]);
    assert!(
        steady(&channels[3], SOUND.right),
        "{:?}",
        &channels[3][..12]
    );
}

#[test]
fn sounds_on_one_channel_add_up_and_a_mono_channel_takes_both_halves() {
    let mut bench = Bench::new(2)
        .play(1, gains(2, &[(0, [1.0, 0.0])]))
        .play(2, gains(2, &[(0, [1.0, 0.0]), (1, [0.5, 0.5])]));
    let channels = bench.render();
    assert!(steady(&channels[0], 2.0 * SOUND.left));
    assert!(steady(&channels[1], f32::midpoint(SOUND.left, SOUND.right)));
}
