// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of GO, the clock, stops, pauses, and panics

mod clips;
mod fading;

use super::*;

#[test]
fn go_fires_the_chain_and_moves_standby_past_it() {
    let workspace = workspace(vec![continuing_after(1, 2.0), audio(2), audio(3)]).apply(GO);
    assert_eq!(workspace.active().len(), 2);
    assert_phase(&workspace, 2, Phase::Scheduled { starts_in: 2.0 });
    assert_eq!(workspace.lists()[0].standby, Some(CueId(3)));
}

#[test]
fn stop_cue_stops_its_target() {
    let workspace = workspace(vec![audio(1), stop_cue(2, 1)])
        .apply(GO)
        .apply(GO);
    assert!(workspace.active().is_empty());
}

#[test]
fn panic_fades_out_and_a_second_panic_stops_at_once() {
    let panicked = workspace(vec![audio(1)]).apply(GO).apply(Command::Panic);
    assert!(panicked.clone().apply(Command::Panic).active().is_empty());
    let halfway = panicked.apply(Command::Tick(PANIC_FADE / 2.0));
    assert!((level(&halfway, 1).unwrap() - SILENCE / 2.0).abs() < 1e-4);
    assert!(halfway.apply(Command::Tick(PANIC_FADE)).active().is_empty());
}

#[test]
fn pause_holds_time_and_fades() {
    let workspace = workspace(vec![continuing(1), fade_out(2)])
        .apply(GO)
        .apply(Command::Pause(true))
        .apply(Command::Tick(2.0));
    assert_eq!(level(&workspace, 1), Some(0.0));
    assert_eq!(workspace.active_of(CueId(1)).unwrap().elapsed, 0.0);
}

#[test]
fn go_skips_broken_cues_in_the_chain() {
    let untargeted = Cue::new(CueId(2), "2", Kind::Fade, "Fade").build();
    let workspace = workspace(vec![continuing(1), untargeted]).apply(GO);
    assert_eq!(running(&workspace), [1]);
}

#[test]
fn stopping_a_cue_cancels_only_the_rest_of_its_chain_that_it_has_yet_to_start() {
    let stopped = |cues: Vec<Cue>| {
        workspace(cues)
            .apply(GO)
            .apply(Command::Tick(1.0))
            .apply(Command::Stop(CueId(1)))
    };
    let unstarted = stopped(vec![continuing_after(1, 2.0), audio(2)]);
    assert!(unstarted.active().is_empty());
    let started = stopped(vec![continuing(1), continuing_after(2, 2.0), audio(3)]);
    assert_eq!(running(&started), [2, 3]);
}

#[test]
fn clock_keeps_time_through_an_hour_of_ticks() {
    let hold = Cue::new(CueId(1), "1", Kind::Wait, "Hold")
        .duration(7200.0)
        .build();
    let workspace = (0..144_000).fold(workspace(vec![hold]).apply(GO), |workspace, _| {
        workspace.apply(Command::Tick(0.025))
    });
    let elapsed = workspace.active_of(CueId(1)).unwrap().elapsed;
    assert!((elapsed - 3600.0).abs() < 1e-6, "{elapsed}");
}
