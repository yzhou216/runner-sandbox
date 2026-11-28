// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of fades, and of the cues that they leave alone or cancel

use super::*;

#[test]
fn fade_moves_the_level_in_a_straight_line_and_stops_its_target_unless_stopped() {
    let workspace = workspace(vec![continuing(1), fade_out(2)])
        .apply(GO)
        .apply(Command::Tick(1.0));
    assert!((level(&workspace, 1).unwrap() - SILENCE / 4.0).abs() < 1e-4);
    // Stopping the fade cue leaves its target at the level reached
    let stopped = workspace
        .clone()
        .apply(Command::Stop(CueId(2)))
        .apply(Command::Tick(5.0));
    assert!((level(&stopped, 1).unwrap() - SILENCE / 4.0).abs() < 1e-4);
    assert!(workspace.apply(Command::Tick(3.0)).active().is_empty());
}

#[test]
fn fade_cue_leaves_a_cue_that_a_panic_is_stopping() {
    let workspace = workspace(vec![audio(1), fade_under(2)])
        .apply(GO)
        .apply(Command::Panic)
        .apply(GO)
        .apply(Command::Tick(PANIC_FADE));
    assert!(workspace.active_of(CueId(1)).is_none());
}

#[test]
fn fade_cue_stopped_in_the_same_tick_does_not_fade() {
    let cues = vec![continuing(1), continued(stop_cue(2, 3)), fade_out(3)];
    let workspace = workspace(cues).apply(GO);
    assert_eq!(workspace.active_of(CueId(1)).unwrap().fade, None);
}

#[test]
fn fade_leaves_a_target_that_has_yet_to_sound() {
    let waiting = pre_waiting(5.0, audio(1));
    let workspace = workspace(vec![waiting, fade_out(2)])
        .apply(GO)
        .apply(Command::Tick(1.0))
        .apply(GO)
        .apply(Command::Tick(4.0));
    assert_eq!(workspace.active_of(CueId(1)).unwrap().fade, None);
}

#[test]
fn fade_that_starts_late_catches_up() {
    let fade = Cue {
        pre_wait: 0.5.into(),
        duration: 1.0.into(),
        ..fade_out(2)
    };
    let workspace = workspace(vec![continuing(1), fade])
        .apply(GO)
        .apply(Command::Tick(2.0));
    assert!(workspace.active_of(CueId(1)).is_none());
}

#[test]
fn cue_faded_out_cancels_the_rest_of_its_chain_that_it_has_yet_to_start() {
    let workspace = workspace(vec![continuing_after(1, 6.0), audio(2), fade_out(3)])
        .apply(GO)
        .apply(GO)
        .apply(Command::Tick(4.0));
    assert!(workspace.active_of(CueId(2)).is_none());
}

#[test]
fn fade_that_stops_its_target_ends_a_loop() {
    let looping = looped(continuing(1));
    let workspace = workspace(vec![looping, fade_out(2)])
        .apply(GO)
        .apply(Command::Tick(4.0));
    assert!(workspace.active().is_empty());
}
