// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of where the sound of a clip with a loop section is, of a clip that
//! a devamp cue has let go on, and of its passes

use super::*;

#[test]
fn clip_has_played_as_far_as_its_sound_has_reached_in_its_file() {
    assert_eq!(looping_clip(false).played_at(4.5, 2.4), 2.5);
    // Three seconds a pass: the sound at 3.0 in the file is a second into a
    // pass, and the clock tells which
    let pass = |position, near| looping_clip(true).played_at(position, near);
    assert_eq!(pass(3.0, 0.5), 1.0);
    assert_eq!(pass(3.0, 3.4), 4.0);
    // Sound just past the loop point, with a clock just short of it, and
    // the other way round
    assert!((pass(2.1, 2.9) - 3.1).abs() < 1e-9);
    assert!((pass(4.9, 3.1) - 2.9).abs() < 1e-9);
    // Each position of the sound maps back to the time it came from
    assert!(
        [0.0, 0.7, 3.0, 4.2, 20.5]
            .iter()
            .all(|&at| (pass(looping_clip(true).position(at), at) - at).abs() < 1e-9)
    );
}

#[test]
fn clip_plays_to_the_end_of_its_loop_then_goes_round_the_loop() {
    let clip = sectioned();
    assert_eq!(clip.position(1.5), 2.5);
    assert_eq!(clip.position(3.5), 4.5);
    assert_eq!(clip.position(4.5), 3.5);
    assert_eq!(clip.position(7.0), 4.0);
    assert_eq!(clip.lasts(), None);
    assert!(
        [0.0, 1.9, 2.0, 3.2, 4.0, 6.9, 21.3].iter().all(|&at| (clip
            .played_at(clip.position(at), at)
            - at)
            .abs()
            < 1e-9)
    );
}

#[test]
fn devamp_lets_a_loop_go_on_at_the_end_of_the_pass_under_way() {
    let clip = sectioned();
    let released = |played| {
        clip.released(played, 0.5)
            .looping
            .and_then(|loop_| loop_.released)
    };
    assert_eq!(released(0.5), Some(4.0));
    assert_eq!(released(4.2), Some(6.0));
    // Too near the end of the pass for the sound to leave the loop there
    assert_eq!(released(5.7), Some(8.0));
    let free = clip.released(4.2, 0.5);
    assert_eq!(free.lasts(), Some(10.0));
    assert_eq!(free.last_pass(), Some(4.0));
    assert_eq!(free.position(5.0), 4.0);
    assert_eq!(free.position(7.5), 6.5);
    assert_eq!(free.played_at(6.5, 7.4), 7.5);
    // Sound still in its last pass while the clock has passed its end
    assert_eq!(free.played_at(4.9, 6.2), 5.9);
    assert_eq!(free.released(4.9, 0.5), free);
}

#[test]
fn clip_makes_passes_through_its_loop_and_then_to_its_end() {
    let clip = sectioned();
    let pass = |from, to, at| Some(Pass { from, to, at });
    assert_eq!(clip.pass(0), pass(1.0, 5.0, 0.0));
    assert_eq!(clip.pass(1), pass(3.0, 5.0, 4.0));
    assert_eq!(clip.pass(7), pass(3.0, 5.0, 16.0));
    let free = clip.released(4.2, 0.5);
    assert_eq!(free.pass(1), pass(3.0, 5.0, 4.0));
    assert_eq!(free.pass(2), pass(5.0, 9.0, 6.0));
    assert_eq!(free.pass(3), None);
    let to_end = Clip { end: 5.0, ..free };
    assert_eq!(to_end.pass(2), None);
    assert_eq!(to_end.lasts(), Some(6.0));
    let once = Clip {
        looping: None,
        ..clip
    };
    assert_eq!(once.pass(0), pass(1.0, 9.0, 0.0));
    assert_eq!(once.pass(1), None);
}

#[test]
fn sound_in_the_last_pass_has_played_to_that_pass_whatever_the_clock_says() {
    let free = sectioned().released(4.2, 0.5);
    assert_eq!(free.played_at(4.9, 100.0), 5.9);
    assert_eq!(free.played_at(3.0, 100.0), 4.0);
    assert_eq!(free.played_at(3.0, 0.0), 2.0);
}
