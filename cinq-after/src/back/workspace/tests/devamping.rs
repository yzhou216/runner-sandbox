// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of devamp cues, which let a cue that loops leave its loop at the
//! end of a pass and play on to its end

use super::*;
use crate::back::{Shown, cue::Seconds};

/// Returns a show whose vamp has played for `played` seconds, with a devamp
/// cue for it up next
fn vamping(played: f64) -> Workspace {
    workspace(vec![vamp(1), devamp_cue(2, 1)])
        .apply(GO)
        .apply(Command::Tick(played))
}

/// Returns the seconds of the clip of cue 1 at which its sound is to leave
/// its loop, once a devamp cue has let it go on
fn released(workspace: &Workspace) -> Option<f64> {
    let active = workspace.active_of(CueId(1)).expect("the vamp plays on");
    active
        .sound()
        .or(active.pending())
        .and_then(|clip| clip.looping)
        .and_then(|section| section.released)
}

fn phase(workspace: &Workspace) -> Phase {
    workspace
        .active_of(CueId(1))
        .expect("the vamp plays on")
        .phase()
}

#[test]
fn devamp_lets_a_loop_play_on_to_its_end_from_the_end_of_the_pass_under_way() {
    let workspace = workspace(vec![vamp(1), devamp_cue(2, 1)])
        .apply(GO)
        .apply(Command::Tick(3.0))
        .apply(GO);
    let active = workspace.active_of(CueId(1)).expect("the vamp plays on");
    assert_eq!(active.phase(), Phase::Running { remaining: 7.0 });
    let ending = workspace.apply(Command::Tick(6.9));
    assert!(ending.active_of(CueId(1)).is_some());
    assert!(ending.apply(Command::Tick(0.2)).active().is_empty());
}

#[test]
fn devamp_leaves_at_the_first_end_of_a_pass_that_lies_more_than_the_notice_ahead() {
    // Loop runs from 2 to 6 seconds, so a pass ends at 6, 10, and 14, and
    // the clip then plays the 4 seconds from the loop end to its own
    let rows = [
        (0.0, 6.0),
        (3.0, 6.0),
        (5.25, 6.0),
        (5.5, 10.0),
        (5.75, 10.0),
        (7.0, 10.0),
        (9.25, 10.0),
        (9.5, 14.0),
    ];
    rows.into_iter().for_each(|(played, leaves)| {
        let devamped = vamping(played).apply(GO);
        assert_eq!(released(&devamped), Some(leaves), "after {played}");
        let remaining = leaves + 4.0 - played;
        assert_eq!(
            phase(&devamped),
            Phase::Running { remaining },
            "after {played}"
        );
    });
}

#[test]
fn loop_that_ends_with_the_clip_ends_the_run_at_the_end_of_the_pass() {
    let to_the_end = Cue {
        loop_end: Seconds::default(),
        ..vamp(1)
    };
    let devamped = |played| {
        workspace(vec![to_the_end.clone(), devamp_cue(2, 1)])
            .apply(GO)
            .apply(Command::Tick(played))
            .apply(GO)
    };
    // Loop runs from 2 to 10 seconds, so a pass ends at 10 and 18
    let near = devamped(3.0);
    assert_eq!(phase(&near), Phase::Running { remaining: 7.0 });
    assert_eq!(near.clone().apply(Command::Tick(6.9)).active().len(), 1);
    assert!(near.apply(Command::Tick(7.0)).active().is_empty());
    let late = devamped(9.75);
    assert_eq!(phase(&late), Phase::Running { remaining: 8.25 });
    assert!(late.apply(Command::Tick(8.25)).active().is_empty());
}

#[test]
fn paused_target_leaves_at_the_end_of_the_pass_after_the_point_where_it_is_paused() {
    let held = |played| {
        workspace(vec![
            vamp(1),
            pause_cue(2, 1),
            devamp_cue(3, 1),
            start_cue(4, 1),
        ])
        .apply(GO)
        .apply(Command::Tick(played))
        .apply(GO)
        .apply(GO)
    };
    let near = held(5.25);
    assert!(near.active_of(CueId(1)).unwrap().paused);
    assert_eq!(phase(&near), Phase::Running { remaining: 4.75 });
    let waiting = near.apply(Command::Tick(30.0));
    assert_eq!(phase(&waiting), Phase::Running { remaining: 4.75 });
    let resumed = waiting.apply(GO);
    assert!(!resumed.active_of(CueId(1)).unwrap().paused);
    assert_eq!(resumed.clone().apply(Command::Tick(4.5)).active().len(), 1);
    assert!(resumed.apply(Command::Tick(4.75)).active().is_empty());
    let late = held(5.5);
    assert_eq!(phase(&late), Phase::Running { remaining: 8.5 });
}

#[test]
fn target_in_its_pre_wait_plays_through_its_loop_once_and_goes_on() {
    let waiting = workspace(vec![pre_waiting(4.0, vamp(1)), devamp_cue(2, 1)])
        .apply(GO)
        .apply(Command::Tick(1.0))
        .apply(GO);
    assert_eq!(phase(&waiting), Phase::PreWait { remaining: 3.0 });
    assert_eq!(released(&waiting), Some(6.0));
    let playing = waiting.apply(Command::Tick(3.0));
    assert_eq!(phase(&playing), Phase::Running { remaining: 10.0 });
}

#[test]
fn target_waiting_in_its_chain_plays_through_its_loop_once_and_goes_on() {
    let waiting = workspace(vec![continuing_after(1, 5.0), vamp(2), devamp_cue(3, 2)])
        .apply(GO)
        .apply(Command::Tick(1.0))
        .apply(GO);
    let vamp = waiting.active_of(CueId(2)).expect("the vamp waits");
    assert_eq!(vamp.phase(), Phase::Scheduled { starts_in: 4.0 });
    let playing = waiting.apply(Command::Tick(4.0));
    let vamp = playing.active_of(CueId(2)).expect("the vamp plays");
    assert_eq!(vamp.phase(), Phase::Running { remaining: 10.0 });
    let ending = playing.apply(Command::Tick(9.75));
    assert!(ending.active_of(CueId(2)).is_some());
    assert!(
        ending
            .apply(Command::Tick(0.25))
            .active_of(CueId(2))
            .is_none()
    );
}

#[test]
fn target_with_no_run_is_left_alone() {
    let missed = workspace(vec![vamp(1), devamp_cue(2, 1)])
        .apply(move_standby(2))
        .apply(GO);
    assert!(missed.active().is_empty());
    let later = missed.apply(move_standby(1)).apply(GO);
    assert_eq!(phase(&later), Phase::Looping);
    assert_eq!(released(&later), None);
    assert_eq!(phase(&later.apply(Command::Tick(100.0))), Phase::Looping);
}

#[test]
fn second_devamp_keeps_the_pass_that_the_first_chose() {
    let twice = workspace(vec![vamp(1), devamp_cue(2, 1), devamp_cue(3, 1)])
        .apply(GO)
        .apply(Command::Tick(3.0))
        .apply(GO)
        .apply(Command::Tick(2.75));
    assert_eq!(phase(&twice), Phase::Running { remaining: 4.25 });
    let again = twice.apply(GO);
    assert_eq!(released(&again), Some(6.0));
    assert_eq!(phase(&again), Phase::Running { remaining: 4.25 });
}

#[test]
fn firing_the_target_again_starts_a_run_that_loops_again() {
    let again = vamping(3.0)
        .apply(GO)
        .apply(Command::Tick(1.0))
        .apply(move_standby(1))
        .apply(GO);
    assert_eq!(phase(&again), Phase::Looping);
    assert_eq!(released(&again), None);
    assert_eq!(phase(&again.apply(Command::Tick(50.0))), Phase::Looping);
}

#[test]
fn cue_that_loops_starts_no_cue_by_auto_follow_devamped_or_not() {
    let followed = workspace(vec![following(vamp(1)), audio(2), devamp_cue(3, 1)])
        .apply(GO)
        .apply(move_standby(3))
        .apply(Command::Tick(3.0))
        .apply(GO);
    assert_eq!(running(&followed), [1]);
    let ended = followed.apply(Command::Tick(10.0));
    assert!(ended.active().is_empty());
}

#[test]
fn devamp_cue_lets_a_video_that_loops_go_on_with_its_picture() {
    let movie = Cue {
        kind: Kind::Video,
        file: "movie.mp4".to_owned(),
        ..vamp(1)
    };
    let devamped = workspace(vec![movie, devamp_cue(2, 1)])
        .apply(GO)
        .apply(Command::Tick(3.0))
        .apply(GO);
    let active = devamped.active_of(CueId(1)).expect("the movie plays on");
    assert_eq!(active.phase(), Phase::Running { remaining: 7.0 });
    let released = match active.shown() {
        Some(Shown::Clip(clip)) => clip.looping.and_then(|section| section.released),
        _ => None,
    };
    assert_eq!(released, Some(6.0));
}
