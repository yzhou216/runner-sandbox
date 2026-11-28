// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of pausing, starting, and restarting a group with the cues that act on it

use super::*;

#[test]
fn pause_cue_on_a_group_holds_every_run_it_would_start() {
    let held = workspace([grouped(), vec![pause_cue(6, 1)]].concat())
        .apply(GO)
        .apply(Command::Tick(1.0))
        .apply(GO)
        .apply(Command::Tick(3.0));
    assert_eq!(running(&held), [1, 2, 3, 4, 5]);
    assert!(all_paused(&held).iter().all(|&paused| paused));
    assert_phase(&held, 5, Phase::Scheduled { starts_in: 3.0 });
    assert_phase(&held, 4, Phase::Scheduled { starts_in: 1.0 });
}

#[test]
fn start_cue_on_a_paused_group_resumes_its_runs_and_fires_nothing() {
    let resumed = workspace([grouped(), vec![pause_cue(6, 1), start_cue(7, 1)]].concat())
        .apply(GO)
        .apply(Command::Tick(1.0))
        .apply(GO)
        .apply(Command::Tick(3.0))
        .apply(GO);
    assert_eq!(running(&resumed), [1, 2, 3, 4, 5]);
    assert!(all_paused(&resumed).iter().all(|&paused| !paused));
    let later = resumed.apply(Command::Tick(1.0));
    assert_phase(&later, 5, Phase::Scheduled { starts_in: 2.0 });
}

#[test]
fn start_cue_on_a_group_that_is_not_paused_fires_it_with_the_cues_it_holds() {
    let fired = workspace(vec![
        start_cue(1, 2),
        group(2),
        within(1, audio(3)),
        within(1, audio(4)),
    ])
    .apply(GO)
    .apply(Command::Tick(1.0));
    assert_eq!(running(&fired), [2, 3, 4]);
    assert_eq!(standby(&fired), Some(2));
}

#[test]
fn go_on_a_group_again_restarts_it_and_cancels_what_the_old_run_would_start() {
    let first = workspace(grouped()).apply(GO).apply(Command::Tick(1.0));
    let old = run_of(&first, 1);
    let again = first.apply(move_standby(1)).apply(GO);
    assert_eq!(running(&again).len(), 5);
    assert!(again.active().iter().all(|active| active.run > old));
    assert_phase(&again, 5, Phase::Scheduled { starts_in: 4.0 });
}

#[test]
fn go_to_cue_on_a_group_puts_it_on_standby() {
    let moved = workspace(vec![goto_cue(1, 2), group(2), within(1, audio(3))]).apply(GO);
    assert_eq!(standby(&moved), Some(2));
}

#[test]
fn fade_and_devamp_cues_cannot_target_a_group() {
    let workspace = workspace(vec![
        group(1),
        within(1, vamp(2)),
        control(3, Kind::Fade, 1),
        devamp_cue(4, 1),
    ]);
    assert_eq!(
        problems(&workspace),
        [(3, Problem::WrongTarget), (4, Problem::NotAudioOrVideo)]
    );
}
