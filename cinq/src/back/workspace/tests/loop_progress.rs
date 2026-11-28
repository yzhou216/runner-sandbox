// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the progress of a cue that loops, which follows where its sound is
//! in its clip

use super::*;

/// Returns the progress of cue 1 in percent at each of the clock times `at`,
/// counted from `from`, where `at` ascends
fn percent(workspace: Workspace, from: f64, at: &[f64]) -> Vec<i32> {
    at.iter()
        .scan((workspace, from), |(workspace, now), &time| {
            *workspace = workspace.clone().apply(Command::Tick(time - *now));
            *now = time;
            let active = workspace.active_of(CueId(1))?;
            Some((active.progress() * 100.0).round() as i32)
        })
        .collect()
}

#[test]
fn progress_runs_to_the_end_of_the_loop_and_back_to_its_start_each_pass() {
    // Loop runs from 2 to 6 seconds of a file of ten
    let show = workspace(vec![vamp(1)]).apply(GO);
    let at = [1.0, 5.0, 5.5, 6.0, 7.0, 9.0, 10.0, 12.0];
    assert_eq!(percent(show, 0.0, &at), [10, 50, 55, 20, 30, 50, 20, 40]);
}

#[test]
fn progress_goes_on_to_the_end_without_going_back_once_a_devamp_cue_lets_go() {
    let show = workspace(vec![vamp(1), devamp_cue(2, 1)])
        .apply(GO)
        .apply(Command::Tick(3.0))
        .apply(GO);
    let at = [5.0, 6.0, 8.0, 9.5];
    assert_eq!(percent(show, 3.0, &at), [50, 60, 80, 95]);
}

#[test]
fn progress_of_a_clip_that_loops_whole_goes_round_as_it_did() {
    let show = workspace(vec![looped(audio(1))]).apply(GO);
    assert_eq!(percent(show, 0.0, &[2.5, 10.0, 12.5]), [25, 0, 25]);
}

#[test]
fn progress_of_a_clip_that_does_not_loop_is_how_much_of_it_has_played() {
    let show = workspace(vec![Cue {
        start: 2.0.into(),
        end: 6.0.into(),
        ..audio(1)
    }])
    .apply(GO);
    assert_eq!(percent(show, 0.0, &[0.0, 1.0, 3.0]), [0, 25, 75]);
}
