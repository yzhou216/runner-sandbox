// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of start cues, which fire their target with its chain or resume it
//! where it is paused

use super::*;
use crate::back::Change;

#[test]
fn start_cue_fires_its_target_and_the_chain_without_moving_standby() {
    let workspace = workspace(vec![
        start_cue(1, 3),
        audio(2),
        continuing_after(3, 2.0),
        audio(4),
        audio(5),
    ])
    .apply(rename(5, "Last"))
    .apply(GO);
    // The start cue takes no time, standby has moved only as GO moves it, and
    // the rename is still the change that undo takes back
    assert_eq!(running(&workspace), [3, 4]);
    assert_eq!(standby(&workspace), Some(2));
    assert_phase(&workspace, 4, Phase::Scheduled { starts_in: 2.0 });
    assert!(matches!(workspace.undoable(), Some(Change::Edited { .. })));
}

#[test]
fn start_cue_fires_a_target_in_another_list_and_leaves_that_standby_and_history() {
    let workspace = lists_of(vec![
        vec![start_cue(1, 3)],
        vec![audio(2), continuing(3), audio(4)],
    ])
    .apply(move_standby_in(1, 3))
    .apply(GO);
    assert_eq!(running(&workspace), [3, 4]);
    assert_eq!(standby_of(&workspace, 1), Some(3));
    assert!(workspace.undoable().is_some());
    assert_eq!(
        standby_of(&workspace.apply(Command::Undo), 1),
        Some(2),
        "the move of that standby is still there to undo"
    );
}

#[test]
fn start_cue_restarts_a_target_that_is_running() {
    let workspace = workspace(vec![audio(1), start_cue(2, 1)])
        .apply(GO)
        .apply(Command::Tick(3.0));
    let before = workspace.active_of(CueId(1)).unwrap().run;
    let restarted = workspace.apply(GO);
    let after = restarted.active_of(CueId(1)).unwrap();
    assert_ne!(after.run, before);
    assert_eq!(after.phase(), Phase::Running { remaining: 10.0 });
    assert_eq!(running(&restarted), [1]);
}

#[test]
fn start_cue_restarting_a_target_cancels_what_the_old_run_had_yet_to_start() {
    let workspace = workspace(vec![continuing_after(1, 5.0), audio(2), start_cue(3, 1)])
        .apply(GO)
        .apply(Command::Tick(1.0))
        .apply(GO);
    // Cue 2 waits for the new run of cue 1 alone
    assert_eq!(running(&workspace), [1, 2]);
    assert_phase(&workspace, 2, Phase::Scheduled { starts_in: 5.0 });
}

#[test]
fn start_cue_resumes_a_paused_target_and_its_followers_instead_of_restarting() {
    let paused = workspace(vec![continuing_after(1, 2.0), audio(2), start_cue(3, 1)])
        .apply(GO)
        .apply(Command::Tick(1.0))
        .apply(Command::Pause(true));
    let run = paused.active_of(CueId(1)).unwrap().run;
    let resumed = paused.apply(GO);
    assert_eq!(running(&resumed), [1, 2]);
    assert_eq!(resumed.active_of(CueId(1)).unwrap().run, run);
    assert!(resumed.active().iter().all(|active| !active.paused));
    let later = resumed.apply(Command::Tick(1.0));
    assert_phase(&later, 1, Phase::Running { remaining: 8.0 });
    assert_phase(&later, 2, Phase::Running { remaining: 10.0 });
}

#[test]
fn start_cue_leaves_the_cues_it_did_not_resume_paused() {
    let paused = workspace(vec![audio(1), audio(2), start_cue(3, 1)])
        .apply(GO)
        .apply(GO)
        .apply(Command::Pause(true));
    let resumed = paused.apply(GO);
    let held = |id| resumed.active_of(CueId(id)).unwrap().paused;
    assert_eq!((held(1), held(2)), (false, true));
}

#[test]
fn start_cue_fires_an_instant_stop_cue_that_acts_on_the_next_tick() {
    let fired = workspace(vec![audio(1), start_cue(2, 3), stop_cue(3, 1)])
        .apply(GO)
        .apply(GO);
    // Stop cue is neither dropped unheard nor acting within the tick
    assert_eq!(running(&fired), [1, 3]);
    assert_eq!(running(&fired.apply(Command::Tick(0.0))), Vec::<u32>::new());
}

#[test]
fn start_cues_that_start_each_other_go_round_once_a_tick() {
    let fired = workspace(vec![start_cue(1, 2), start_cue(2, 1)]).apply(GO);
    let round = |workspace: &Workspace| {
        workspace
            .active()
            .iter()
            .map(|active| (active.id.0, active.run))
            .collect::<Vec<_>>()
    };
    assert_eq!(round(&fired), [(2, 1)]);
    let second = fired.apply(Command::Tick(0.0));
    assert_eq!(round(&second), [(1, 2)]);
    assert_eq!(round(&second.apply(Command::Tick(0.0))), [(2, 3)]);
}

#[test]
fn start_cue_fires_its_target_as_far_along_as_the_tick_passed_its_pre_wait() {
    let waiting = pre_waiting(0.5, start_cue(1, 2));
    let workspace = workspace(vec![waiting, continuing_after(2, 2.0), audio(3)])
        .apply(GO)
        .apply(Command::Tick(2.0));
    assert_eq!(running(&workspace), [2, 3]);
    assert_phase(&workspace, 2, Phase::Running { remaining: 8.5 });
    assert_phase(&workspace, 3, Phase::Scheduled { starts_in: 0.5 });
}

#[test]
fn start_cue_fires_nothing_where_its_target_cannot_play() {
    let untargeted = Cue::new(CueId(2), "2", Kind::Fade, "Fade").build();
    let workspace = workspace(vec![start_cue(1, 2), untargeted]).apply(GO);
    assert!(workspace.active().is_empty());
}

#[test]
fn cues_ahead_take_in_what_start_cues_fire_with_their_delays() {
    let waiting = pre_waiting(1.0, start_cue(1, 3));
    let workspace = lists_of(vec![
        vec![waiting, audio(2)],
        vec![continuing_after(3, 2.0), audio(4)],
    ]);
    // Next GO of the second list fires cue 3 as well
    assert_eq!(
        ahead(&workspace),
        [(1, 0.0), (3, 1.0), (4, 3.0), (3, 0.0), (4, 2.0)]
    );
    // Once GO has fired the start cue, its target stays ahead until it acts
    let fired = workspace.apply(GO).apply(Command::Tick(0.5));
    assert_eq!(&ahead(&fired)[..3], [(1, -0.5), (3, 0.5), (4, 2.5)]);
}

#[test]
fn cues_ahead_leave_out_what_a_start_cue_only_resumes() {
    let workspace = workspace(vec![audio(1), start_cue(2, 1), audio(3)])
        .apply(GO)
        .apply(Command::Pause(true));
    assert_eq!(ahead(&workspace), [(2, 0.0)]);
}

#[test]
fn cues_reached_list_what_two_start_cues_fire_together_once() {
    let workspace = lists_of(vec![
        vec![following(start_cue(1, 3)), start_cue(2, 3)],
        vec![audio(3), audio(4)],
    ]);
    let fired = [1, 2].map(|id| (workspace.cue(CueId(id)).unwrap(), 0.0));
    let reached: Vec<u32> = workspace
        .reached(fired.to_vec())
        .into_iter()
        .map(|(cue, _)| cue.id.0)
        .collect();
    assert_eq!(reached, [1, 2, 3]);
}

#[test]
fn cues_reached_keep_the_smallest_delay_of_the_start_cues_that_fire_them() {
    let workspace = lists_of(vec![
        vec![
            pre_waiting(5.0, start_cue(1, 3)),
            pre_waiting(1.0, start_cue(2, 3)),
        ],
        vec![audio(3), audio(4)],
    ]);
    let fired = [1, 2].map(|id| (workspace.cue(CueId(id)).unwrap(), 0.0));
    let reached: Vec<(u32, f32)> = workspace
        .reached(fired.to_vec())
        .into_iter()
        .map(|(cue, delay)| (cue.id.0, delay))
        .collect();
    assert_eq!(reached, [(1, 0.0), (2, 0.0), (3, 1.0)]);
}

#[test]
fn cues_reached_take_the_smaller_delay_that_a_later_round_gives_them() {
    // Cue 1 reaches cue 5 after its pre-wait of 5, and cue 2 reaches the
    // start cue 3, which reaches cue 5 at once, a round later
    let workspace = lists_of(vec![
        vec![pre_waiting(5.0, start_cue(1, 5)), start_cue(2, 3)],
        vec![start_cue(3, 5)],
        vec![audio(5)],
    ]);
    let fired = [1, 2].map(|id| (workspace.cue(CueId(id)).unwrap(), 0.0));
    let reached: Vec<(u32, f32)> = workspace
        .reached(fired.to_vec())
        .into_iter()
        .map(|(cue, delay)| (cue.id.0, delay))
        .collect();
    assert_eq!(reached, [(1, 0.0), (2, 0.0), (5, 0.0), (3, 0.0)]);
}

#[test]
fn cues_ahead_follow_a_chain_of_thousands_of_start_cues() {
    // Each start cue fires the next, as a show file may chain them
    let cues = (1..=3000)
        .map(|id| start_cue(id, id + 1))
        .chain([audio(3001)])
        .collect();
    let ahead = ahead(&workspace(cues));
    assert_eq!(ahead.len(), 3001);
    assert!(ahead.iter().all(|&(_, delay)| delay == 0.0));
}
