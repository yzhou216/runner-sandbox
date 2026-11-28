// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of what a devamp cue needs to play, and of where it sits among other
//! cues

use super::*;

#[test]
fn devamp_cue_needs_a_target_that_loops() {
    let wait = Cue::new(CueId(5), "5", Kind::Wait, "Wait").build();
    let show = workspace(vec![
        audio(1),
        vamp(2),
        devamp_cue(3, 1),
        devamp_cue(4, 2),
        wait,
        devamp_cue(6, 5),
        devamp_cue(7, 7),
        devamp_cue(8, 9),
    ]);
    assert_eq!(
        problems(&show),
        [
            (3, Problem::NotLooping),
            (6, Problem::NotAudioOrVideo),
            (7, Problem::WrongTarget),
            (8, Problem::NoTarget),
        ]
    );
}

#[test]
fn loop_that_starts_at_or_after_its_end_is_a_problem_and_go_skips_the_cue() {
    let looping = |loop_start: f32, loop_end: &str| Cue {
        loop_start: loop_start.into(),
        loop_end: loop_end.to_owned().into(),
        ..vamp(1)
    };
    let problem = |cue: Cue| {
        let show = workspace(vec![cue]);
        let cue = show.cue(CueId(1)).expect("the cue is there");
        show.problem(cue).map(|problem| problem.to_string())
    };
    let empty = Some("loop starts at or after its end".to_owned());
    assert_eq!(problem(looping(6.0, "2")), empty);
    assert_eq!(problem(looping(4.0, "4")), empty);
    assert_eq!(problem(looping(12.0, "")), empty);
    assert_eq!(problem(looping(0.0, "")), None);
    let not_looping = Cue {
        looping: false,
        ..looping(6.0, "2")
    };
    assert_eq!(problem(not_looping), None);
    let skipped = workspace(vec![looping(6.0, "2"), audio(2)]).apply(GO);
    assert!(skipped.active().is_empty());
    assert_eq!(standby(&skipped), Some(2));
}

#[test]
fn go_skips_a_devamp_cue_whose_target_does_not_loop() {
    let skipped = workspace(vec![audio(1), devamp_cue(2, 1), audio(3)])
        .apply(move_standby(2))
        .apply(GO);
    assert!(skipped.active().is_empty());
    assert_eq!(standby(&skipped), Some(3));
}

#[test]
fn devamp_cue_lets_a_loop_of_another_list_go_on() {
    let show = lists_of(vec![vec![vamp(1)], vec![devamp_cue(2, 1)]])
        .apply(GO)
        .apply(Command::Tick(3.0))
        .apply(Command::Go { list: 1 });
    let active = show.active_of(CueId(1)).expect("the vamp plays on");
    assert_eq!(active.phase(), Phase::Running { remaining: 7.0 });
}

#[test]
fn devamp_cue_in_a_chain_lets_go_after_its_pre_wait_from_where_the_target_is_then() {
    let chain = |pre_wait: f32| {
        workspace(vec![
            continued(vamp(1)),
            pre_waiting(pre_wait, devamp_cue(2, 1)),
        ])
        .apply(GO)
    };
    let phase = |workspace: &Workspace| workspace.active_of(CueId(1)).unwrap().phase();
    let early = chain(3.0);
    assert_eq!(
        phase(&early.clone().apply(Command::Tick(2.75))),
        Phase::Looping
    );
    assert_eq!(
        phase(&early.apply(Command::Tick(3.0))),
        Phase::Running { remaining: 7.0 }
    );
    let late = chain(5.75).apply(Command::Tick(5.75));
    assert_eq!(phase(&late), Phase::Running { remaining: 8.25 });
}

#[test]
fn stop_and_fade_cues_act_on_a_devamped_run_as_on_any_run() {
    let devamped = |cue: Cue| {
        workspace(vec![vamp(1), devamp_cue(2, 1), cue])
            .apply(GO)
            .apply(Command::Tick(3.0))
            .apply(GO)
            .apply(GO)
    };
    assert!(devamped(stop_cue(3, 1)).active_of(CueId(1)).is_none());
    let fading = devamped(fade_out(3)).apply(Command::Tick(2.0));
    assert!((level(&fading, 1).unwrap() + 30.0).abs() < 1e-3);
    assert!(
        fading
            .apply(Command::Tick(2.0))
            .active_of(CueId(1))
            .is_none()
    );
    let under = devamped(fade_under(3)).apply(Command::Tick(5.0));
    assert!((level(&under, 1).unwrap() + 18.0).abs() < 1e-3);
    let active = under.active_of(CueId(1)).expect("the vamp plays on");
    assert_eq!(active.phase(), Phase::Running { remaining: 2.0 });
}

#[test]
fn undo_takes_back_a_change_to_the_loop_start_or_the_loop_end() {
    let edited = workspace(vec![vamp(1)])
        .apply(edit(1, Edit::LoopStart("3".to_owned().into())))
        .apply(edit(1, Edit::LoopEnd("5".to_owned().into())));
    let loop_of = |workspace: &Workspace| {
        let cue = workspace.cue(CueId(1)).expect("the vamp is there");
        (cue.loop_start.time(), cue.loop_end.time())
    };
    assert_eq!(loop_of(&edited), (Some(3.0), Some(5.0)));
    let once = edited.apply(Command::Undo);
    assert_eq!(loop_of(&once), (Some(3.0), Some(6.0)));
    assert_eq!(loop_of(&once.apply(Command::Undo)), (Some(2.0), Some(6.0)));
}

#[test]
fn loop_shorter_than_a_millisecond_is_a_problem_and_go_skips_the_cue() {
    let brief = Cue {
        loop_start: 2.0.into(),
        loop_end: "2.0005".to_owned().into(),
        ..vamp(1)
    };
    let show = workspace(vec![brief, audio(2)]);
    let cue = show.cue(CueId(1)).expect("the cue is there");
    let problem = show.problem(cue).map(|problem| problem.to_string());
    assert_eq!(
        problem.as_deref(),
        Some("loop is shorter than a millisecond")
    );
    let skipped = show.apply(GO);
    assert!(skipped.active().is_empty());
    assert_eq!(standby(&skipped), Some(2));
}
