// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of pause cues, which hold their target and the cues that it would
//! start

use super::*;

/// Returns a show whose cue 1 plays under a fade, with cue 2 waiting for
/// its post-wait, after a pause cue has held them
fn held() -> Workspace {
    workspace(vec![
        continuing_after(1, 6.0),
        audio(2),
        fade_under(3),
        pause_cue(4, 1),
        start_cue(5, 1),
    ])
    .apply(GO)
    .apply(GO)
    .apply(Command::Tick(1.0))
    .apply(GO)
}

#[test]
fn pause_cue_holds_a_playing_cue_its_fade_and_the_cue_waiting_on_its_post_wait() {
    let held = held().apply(Command::Tick(3.0));
    let held_level = level(&held, 1).unwrap();
    assert!((held_level + 3.6).abs() < 1e-4, "{held_level}");
    assert_phase(&held, 1, Phase::Running { remaining: 9.0 });
    assert_phase(&held, 2, Phase::Scheduled { starts_in: 5.0 });
    assert!(held.active_of(CueId(1)).unwrap().paused);
    assert!(held.active_of(CueId(2)).unwrap().paused);
}

#[test]
fn start_cue_resumes_what_a_pause_cue_held() {
    let resumed = held().apply(Command::Tick(3.0)).apply(GO);
    assert!(!resumed.active_of(CueId(1)).unwrap().paused);
    let later = resumed.apply(Command::Tick(1.0));
    let moved = level(&later, 1).unwrap();
    assert!((moved + 7.2).abs() < 1e-4, "{moved}");
    assert_phase(&later, 2, Phase::Scheduled { starts_in: 4.0 });
}

#[test]
fn pause_cue_holds_a_cue_in_its_pre_wait_and_a_cue_that_loops() {
    let waiting = pre_waiting(4.0, audio(1));
    let looping = looped(audio(2));
    let held = workspace(vec![waiting, looping, pause_cue(3, 1), pause_cue(4, 2)])
        .apply(GO)
        .apply(GO)
        .apply(Command::Tick(1.0))
        .apply(GO)
        .apply(GO)
        .apply(Command::Tick(5.0));
    assert_phase(&held, 1, Phase::PreWait { remaining: 3.0 });
    assert_phase(&held, 2, Phase::Looping);
    assert_eq!(held.active_of(CueId(2)).unwrap().progress(), 0.1);
}

#[test]
fn pause_cue_whose_target_has_no_run_does_nothing() {
    let workspace = workspace(vec![audio(1), pause_cue(2, 3), audio(3)])
        .apply(move_standby(1))
        .apply(GO)
        .apply(move_standby(2))
        .apply(GO);
    assert_eq!(running(&workspace), [1]);
    assert!(!workspace.active_of(CueId(1)).unwrap().paused);
}

#[test]
fn pause_cue_holds_a_cue_that_a_start_cue_has_fired_before_it_acts() {
    let workspace = lists_of(vec![
        vec![
            audio(1),
            Cue {
                advance: Advance::Continue,
                ..start_cue(2, 4)
            },
            pause_cue(3, 4),
        ],
        vec![stop_cue(4, 1)],
    ])
    .apply(GO)
    .apply(GO)
    .apply(Command::Tick(0.025));
    // Stop cue waits, paused, for something to resume it
    assert_eq!(running(&workspace), [1, 4]);
    assert!(workspace.active_of(CueId(4)).unwrap().paused);
    let resumed = workspace
        .apply(Command::Pause(false))
        .apply(Command::Tick(0.025));
    assert_eq!(running(&resumed), Vec::<u32>::new());
}

#[test]
fn pause_cue_holds_the_fade_that_a_fade_cue_is_making_until_it_resumes() {
    let fading = workspace(vec![audio(1), fade_out(2), pause_cue(3, 2)])
        .apply(GO)
        .apply(GO)
        .apply(Command::Tick(1.0))
        .apply(GO);
    let held = fading.apply(Command::Tick(2.0));
    assert_eq!(level(&held, 1), Some(-15.0));
    assert_phase(&held, 2, Phase::Running { remaining: 3.0 });
    let resumed = held
        .apply(Command::Resume(CueId(2)))
        .apply(Command::Tick(1.0));
    assert_eq!(level(&resumed, 1), Some(-30.0));
    assert_eq!(
        running(&resumed.apply(Command::Tick(3.0))),
        Vec::<u32>::new()
    );
}
