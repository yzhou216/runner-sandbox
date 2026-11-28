// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of editing cues that are running

use super::*;

#[test]
fn level_edit_leaves_a_cue_that_a_panic_is_stopping() {
    let panicked = workspace(vec![audio(1)]).apply(GO).apply(Command::Panic);
    let edited = panicked.clone().apply(edit(1, Edit::Level(-3.0)));
    let halfway =
        |workspace: &Workspace| level(&workspace.clone().apply(Command::Tick(PANIC_FADE / 2.0)), 1);
    assert_eq!(halfway(&edited), halfway(&panicked));
    assert!(edited.apply(Command::Tick(PANIC_FADE)).active().is_empty());
}

#[test]
fn edit_that_changes_nothing_leaves_a_running_cue_playing_and_nothing_to_undo() {
    // Text fields send their text again on keys that change nothing, such
    // as Backspace at the start of the field
    let workspace = workspace(vec![audio(1)])
        .apply(GO)
        .apply(edit(1, Edit::File("sound.ogg".to_owned())));
    let cue = workspace.cue(CueId(1)).unwrap();
    assert!(workspace.active_of(CueId(1)).is_some());
    assert_eq!(workspace.problem(cue), None);
    assert!(workspace.undoable().is_none());
}

#[test]
fn choosing_the_file_that_a_running_cue_plays_leaves_it_playing() {
    let unnamed = Cue {
        name: "New Audio cue".to_owned(),
        ..audio(1)
    };
    let running = workspace(vec![unnamed]).apply(GO);
    let chosen = running
        .cue(CueId(1))
        .unwrap()
        .playing("sound.ogg".to_owned());
    let workspace = running
        .apply(edit(1, chosen))
        .apply(edit(1, Edit::File(" sound.ogg\n".to_owned())));
    let cue = workspace.cue(CueId(1)).unwrap();
    assert_eq!(cue.name, "sound");
    assert_eq!(workspace.problem(cue), None);
    assert!(workspace.active_of(CueId(1)).is_some());
}

#[test]
fn editing_a_running_cue_leaves_the_run_under_way() {
    let workspace = workspace(vec![audio(1)])
        .apply(GO)
        .apply(Command::Tick(3.0))
        .apply(edit(1, Edit::PreWait(10.0.into())));
    let active = workspace.active_of(CueId(1)).unwrap();
    assert_eq!(active.phase(), Phase::Running { remaining: 7.0 });
}

#[test]
fn editing_a_fade_cue_in_its_pre_wait_leaves_the_fade_it_will_make() {
    let fade = pre_waiting(2.0, fade_out(2));
    let workspace = workspace(vec![continuing(1), fade])
        .apply(GO)
        .apply(Command::Tick(1.0))
        .apply(edit(2, Edit::Level(-10.0)))
        .apply(edit(2, Edit::StopTarget(false)))
        .apply(Command::Tick(1.0));
    let fade = workspace.active_of(CueId(1)).unwrap().fade.unwrap();
    assert_eq!((fade.to, fade.then_stop), (-10.0, true));
}

#[test]
fn editing_the_clip_of_a_running_cue_waits_for_its_next_run() {
    let workspace = workspace(vec![looped(audio(1))])
        .apply(GO)
        .apply(edit(1, Edit::Loop(false)))
        .apply(Command::Tick(30.0));
    let active = workspace.active_of(CueId(1)).expect("the loop plays on");
    assert_eq!(active.phase(), Phase::Looping);
}

#[test]
fn level_edit_leaves_a_fade_that_leaves_its_target_playing_running() {
    let fading = workspace(vec![continuing(1), fade_under(2)])
        .apply(GO)
        .apply(Command::Tick(1.0));
    assert!(
        fading
            .active_of(CueId(1))
            .is_some_and(|active| active.fade.is_some())
    );
    let edited = fading.apply(edit(1, Edit::Level(-3.0)));
    assert!(
        edited
            .active_of(CueId(1))
            .is_some_and(|active| active.fade.is_some())
    );
    let active = edited.apply(Command::Tick(5.0));
    let active = active.active_of(CueId(1)).expect("the cue plays on");
    assert_eq!((active.level, active.fade.is_none()), (-18.0, true));
}

#[test]
fn level_edit_of_a_cue_that_is_not_running_waits_for_its_run() {
    let edited = workspace(vec![audio(1), audio(2)]).apply(edit(1, Edit::Level(-3.0)));
    assert!(edited.active().is_empty());
    assert_eq!(level(&edited.apply(GO), 1), Some(-3.0));
}
