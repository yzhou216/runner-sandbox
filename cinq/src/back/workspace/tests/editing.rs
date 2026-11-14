// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of adding, deleting, editing, and loading cues

use super::*;

/// Returns the command that records that the sound file of cue `id` is
/// not found
fn not_found(id: u32) -> Command {
    loaded(id, "sound.ogg", Media::Failed("file not found".to_owned()))
}

#[test]
fn show_mode_blocks_edits_but_not_playback() {
    let workspace = workspace(vec![audio(1), audio(2)])
        .apply(Command::ShowMode(true))
        .apply(rename(1, "Renamed"))
        .apply(moving(1, 1))
        .apply(GO);
    assert_eq!(workspace.cue(CueId(1)).unwrap().name, "Sound");
    assert_eq!(ids(&workspace), [1, 2]);
    assert_eq!(workspace.active().len(), 1);
}

#[test]
fn new_cue_before_another_targets_the_cue_it_follows() {
    let workspace =
        workspace(vec![audio(1), audio(2)]).apply(add(0, Kind::Fade, Place::Before(CueId(2))));
    assert_eq!(ids(&workspace), [1, 3, 2]);
    assert_eq!(workspace.lists()[0].cues[1].target, Some(CueId(1)));
}

#[test]
fn cue_added_waits_on_standby_only_at_the_end_of_a_list_played_to_its_end() {
    let played = workspace(vec![audio(1)]).apply(GO);
    assert_eq!(standby(&played), None);
    let id = played.next_cue_id().expect("identities remain").0;
    let above = played
        .clone()
        .apply(add(0, Kind::Audio, Place::Before(CueId(1))));
    assert_eq!(standby(&above), None);
    let below = played.apply(add(0, Kind::Audio, Place::After(CueId(1))));
    assert_eq!(standby(&below), Some(id));
    let waiting = workspace(vec![audio(1), audio(2)]).apply(add(0, Kind::Audio, Place::End));
    assert_eq!(standby(&waiting), Some(1));
}

#[test]
fn moved_cue_stops_at_either_end_of_its_list_and_keeps_standby() {
    let shown = workspace(vec![audio(1), audio(2), audio(3)]);
    let down = shown.clone().apply(moving(1, 5));
    assert_eq!(ids(&down), [2, 3, 1]);
    assert_eq!(standby(&down), Some(1));
    assert_eq!(ids(&shown.apply(moving(3, -1))), [1, 3, 2]);
}

#[test]
fn loading_ignores_a_file_that_has_been_replaced() {
    let replaced = workspace(vec![audio(1)]).apply(edit(1, Edit::File("other.ogg".to_owned())));
    let stale = replaced.clone().apply(loaded(1, "sound.ogg", sound(10.0)));
    let cue = stale.cue(CueId(1)).unwrap();
    assert_eq!(stale.problem(cue), Some(Problem::NotLoaded));
    // Failure of the file that was replaced leaves the cue playing
    let playing = replaced
        .apply(loaded(1, "other.ogg", sound(10.0)))
        .apply(GO)
        .apply(not_found(1));
    assert!(playing.active_of(CueId(1)).is_some());
}

#[test]
fn new_cues_are_numbered_after_the_highest_number_of_any_list() {
    let lists = vec![
        CueList::new("Main", vec![audio(1), audio(7)]),
        CueList::new("Preshow", Vec::new()),
    ];
    let workspace = Workspace::new(lists).apply(add(1, Kind::Memo, Place::End));
    assert_eq!(workspace.lists()[1].cues[0].number, "8");
}

#[test]
fn level_edit_leaves_a_cue_that_a_panic_is_stopping() {
    let workspace = workspace(vec![audio(1)])
        .apply(GO)
        .apply(Command::Panic)
        .apply(edit(1, Edit::Level(-3.0)))
        .apply(Command::Tick(PANIC_FADE));
    assert!(workspace.active().is_empty());
}

#[test]
fn run_whose_sound_cannot_start_ends_and_its_chain_plays_on() {
    let workspace = workspace(vec![continuing_after(1, 2.0), audio(2)])
        .apply(GO)
        .apply(not_found(1));
    assert_eq!(running(&workspace), [2]);
    let cue = workspace.cue(CueId(1)).unwrap();
    assert_eq!(
        workspace.problem(cue),
        Some(Problem::LoadFailed("file not found"))
    );
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
fn deleting_the_last_cue_on_standby_moves_standby_past_the_end() {
    let workspace = workspace(vec![audio(1), audio(2)])
        .apply(move_standby(2))
        .apply(delete(&[2]));
    assert_eq!(workspace.lists()[0].standby, None);
}

#[test]
fn new_numbers_read_spaced_and_large_numbers_exactly_and_do_not_overflow() {
    let after = |number: &str| {
        let numbered = Cue {
            number: number.to_owned(),
            ..audio(1)
        };
        let added = workspace(vec![numbered]).apply(add(0, Kind::Memo, Place::End));
        added.lists()[0].cues[1].number.clone()
    };
    assert_eq!(after(" 12 "), "13");
    assert_eq!(after("16777217"), "16777218");
    assert_eq!(after("inf"), u64::MAX.to_string());
}

#[test]
fn files_that_could_not_be_played_are_listed_for_another_try() {
    let workspace = workspace(vec![audio(1), audio(2)]).apply(not_found(1));
    let listed: Vec<CueId> = workspace.failed().map(|cue| cue.id).collect();
    assert_eq!(listed, [CueId(1)]);
}

#[test]
fn editing_a_fade_cue_in_its_pre_wait_leaves_the_fade_it_will_make() {
    let fade = Cue {
        pre_wait: 2.0.into(),
        ..fade_out(2)
    };
    let workspace = workspace(vec![continuing(1), fade])
        .apply(GO)
        .apply(Command::Tick(1.0))
        .apply(edit(2, Edit::Level(-10.0)))
        .apply(edit(2, Edit::StopTarget(false)))
        .apply(Command::Tick(1.0));
    let fade = workspace.active_of(CueId(1)).unwrap().fade.unwrap();
    assert_eq!((fade.to, fade.then_stop), (SILENCE, true));
}

#[test]
fn editing_the_clip_of_a_running_cue_waits_for_its_next_run() {
    let workspace = workspace(vec![Cue {
        looping: true,
        ..audio(1)
    }])
    .apply(GO)
    .apply(edit(1, Edit::Loop(false)))
    .apply(Command::Tick(30.0));
    let active = workspace.active_of(CueId(1)).expect("the loop plays on");
    assert_eq!(active.phase(), Phase::Looping);
}

#[test]
fn level_edit_takes_over_from_a_fade_that_leaves_its_target_playing() {
    let fading = workspace(vec![continuing(1), fade_under(2)])
        .apply(GO)
        .apply(Command::Tick(1.0));
    assert!(
        fading
            .active_of(CueId(1))
            .is_some_and(|active| active.fade.is_some())
    );
    let edited = fading
        .apply(edit(1, Edit::Level(-3.0)))
        .apply(Command::Tick(1.0));
    let active = edited.active_of(CueId(1)).expect("the cue plays on");
    assert_eq!((active.level, active.fade.is_none()), (-3.0, true));
}

#[test]
fn level_edit_of_a_cue_that_is_not_running_waits_for_its_run() {
    let edited = workspace(vec![audio(1), audio(2)]).apply(edit(1, Edit::Level(-3.0)));
    assert!(edited.active().is_empty());
    assert_eq!(level(&edited.apply(GO), 1), Some(-3.0));
}
