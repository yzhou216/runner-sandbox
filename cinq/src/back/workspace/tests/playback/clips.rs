// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the clips that audio cues play, loops, and progress

use super::*;
use crate::back::Active;

#[test]
fn audio_cue_plays_from_its_start_to_its_end() {
    let trimmed = Cue {
        start: 2.0.into(),
        end: 5.0.into(),
        ..audio(1)
    };
    let workspace = workspace(vec![trimmed]).apply(GO).apply(Command::Tick(2.0));
    let active = workspace.active_of(CueId(1)).unwrap();
    assert_eq!(active.phase(), Phase::Running { remaining: 1.0 });
    assert!(workspace.apply(Command::Tick(1.0)).active().is_empty());
}

#[test]
fn audio_cue_that_starts_at_or_after_its_end_cannot_play() {
    let backwards = Cue {
        start: 6.0.into(),
        end: 3.0.into(),
        ..audio(1)
    };
    let workspace = workspace(vec![backwards]);
    assert_eq!(problems(&workspace), [(1, Problem::EmptyClip)]);
    assert!(workspace.apply(GO).active().is_empty());
}

#[test]
fn audio_cue_that_starts_past_the_end_of_its_file_cannot_play() {
    let beyond = Cue {
        start: 9999.0.into(),
        ..audio(1)
    };
    let workspace = workspace(vec![beyond]);
    assert_eq!(problems(&workspace), [(1, Problem::PastFile)]);
    assert!(workspace.apply(GO).active().is_empty());
}

#[test]
fn audio_cue_that_loops_plays_until_something_stops_it() {
    let workspace = workspace(vec![looped(audio(1))])
        .apply(GO)
        .apply(Command::Tick(25.0));
    let active = workspace.active_of(CueId(1)).expect("the loop plays on");
    assert_eq!(active.phase(), Phase::Looping);
    assert!((active.progress() - 0.5).abs() < 1e-6);
    assert!(workspace.apply(Command::Stop(CueId(1))).active().is_empty());
}

#[test]
fn audio_cue_that_loops_leaves_the_cue_it_would_follow_to_go() {
    let workspace = workspace(vec![looped(following(audio(1))), audio(2)]).apply(GO);
    assert_eq!(running(&workspace), [1]);
    assert_eq!(standby(&workspace), Some(2));
}

#[test]
fn progress_waits_out_the_pre_wait_and_then_counts_the_clip_played() {
    let waiting = workspace(vec![pre_waiting(4.0, audio(1))]).apply(GO);
    let progress = |workspace: &Workspace| workspace.active_of(CueId(1)).map(Active::progress);
    assert_eq!(progress(&waiting), Some(0.0));
    let in_wait = waiting.clone().apply(Command::Tick(2.0));
    assert_eq!(progress(&in_wait), Some(0.0));
    let halfway = waiting.apply(Command::Tick(9.0));
    assert_eq!(progress(&halfway), Some(0.5));
}

#[test]
fn progress_of_a_picture_that_shows_until_stopped_is_whole() {
    let shown = workspace(vec![picture(1)]).apply(GO);
    let active = shown.active_of(CueId(1)).expect("the picture shows");
    assert_eq!((active.phase(), active.progress()), (Phase::Showing, 1.0));
}
