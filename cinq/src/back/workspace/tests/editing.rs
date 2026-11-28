// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of adding, deleting, moving, numbering, and loading cues

mod running;

use super::*;

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
fn new_fade_cue_targets_the_picture_or_video_cue_it_follows() {
    [picture(1), video(1)].into_iter().for_each(|cue| {
        let kind = cue.kind;
        let workspace = workspace(vec![cue]).apply(add(0, Kind::Fade, Place::After(CueId(1))));
        assert_eq!(
            workspace.lists()[0].cues[1].target,
            Some(CueId(1)),
            "{kind:?}"
        );
    });
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
    assert_eq!(ids(&shown.clone().apply(moving(3, -1))), [1, 3, 2]);
    // A distance as large as a number can hold does not overflow
    assert_eq!(ids(&shown.clone().apply(moving(2, isize::MAX))), [1, 3, 2]);
    assert_eq!(ids(&shown.apply(moving(2, isize::MIN))), [2, 1, 3]);
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
fn deleting_the_last_cue_on_standby_moves_standby_past_the_end() {
    let workspace = workspace(vec![audio(1), audio(2)])
        .apply(move_standby(2))
        .apply(delete(&[2]));
    assert_eq!(workspace.lists()[0].standby, None);
}

#[test]
fn deleting_cues_together_moves_standby_to_the_first_cue_that_stays() {
    let workspace = workspace(vec![audio(1), audio(2), audio(3), audio(4), audio(5)])
        .apply(move_standby(2))
        .apply(delete(&[4, 2, 3]));
    assert_eq!(ids(&workspace), [1, 5]);
    assert_eq!(standby(&workspace), Some(5));
    let undone = workspace.apply(Command::Undo);
    assert_eq!(
        (ids(&undone), standby(&undone)),
        (vec![1, 2, 3, 4, 5], Some(2))
    );
}

#[test]
fn deleting_cues_of_two_lists_together_keeps_each_standby_in_its_list() {
    let workspace = lists_of(vec![
        vec![audio(1), audio(2)],
        vec![audio(3), audio(4), audio(5)],
    ])
    .apply(delete(&[2, 3, 4]));
    let standbys: Vec<Option<CueId>> = workspace.lists().iter().map(|list| list.standby).collect();
    assert_eq!(standbys, [Some(CueId(1)), Some(CueId(5))]);
}

/// Returns the number of the memo cue that goes to the end after `others`
/// follow a cue 1 that holds `number`
fn number_added_after(number: &str, others: Vec<Cue>) -> String {
    let numbered = Cue {
        number: number.to_owned(),
        ..audio(1)
    };
    let added = workspace([vec![numbered], others].concat()).apply(add(0, Kind::Memo, Place::End));
    added.cues().last().expect("a cue was added").number.clone()
}

#[test]
fn new_numbers_read_spaced_and_large_numbers_exactly_and_do_not_overflow() {
    let after = |number: &str| number_added_after(number, Vec::new());
    assert_eq!(after(" 12 "), "13");
    assert_eq!(after("16777217"), "16777218");
    assert_eq!(after("1e300"), u64::MAX.to_string());
}

#[test]
fn numbers_that_are_words_for_infinity_or_not_a_number_are_text_and_not_counted() {
    ["inf", "-inf", "Infinity", "INF", "NaN"]
        .into_iter()
        .for_each(|word| assert_eq!(number_added_after(word, vec![audio(2)]), "3", "{word}"));
}

#[test]
fn files_that_were_read_are_listed_for_another_look_unless_their_cues_run() {
    let workspace = workspace(vec![audio(1), audio(2), audio(3)])
        .apply(not_found(1))
        .apply(GO)
        .apply(GO);
    let listed: Vec<CueId> = workspace.to_recheck().map(|cue| cue.id).collect();
    assert_eq!(workspace.active().len(), 1, "the second GO fires cue 2");
    assert_eq!(listed, [CueId(1), CueId(3)]);
}

#[test]
fn commands_for_cues_and_lists_that_are_not_there_change_nothing() {
    let gone = CueId(9);
    let commands = [
        Command::Go { list: 9 },
        Command::Stop(gone),
        Command::Start(gone),
        Command::Hold(gone),
        Command::Resume(gone),
        Command::GoTo(gone),
        Command::Standby {
            list: 9,
            cue: Some(CueId(1)),
        },
        Command::Add {
            list: 9,
            kind: Kind::Memo,
            at: Place::End,
            count: 1,
        },
        Command::Paste {
            list: 9,
            cues: vec![audio(1)],
            at: Place::End,
        },
        delete(&[9]),
        Command::Cut(vec![gone]),
        moving(9, 1),
        Command::Indent(vec![gone]),
        Command::Outdent(vec![gone]),
        collapse(&[9], true),
        rename(9, "Gone"),
        Command::Heard {
            cue: gone,
            run: 1,
            played: 1.0,
        },
        loaded(9, "sound.ogg", sound(10.0)),
        Command::Undo,
        Command::Redo,
        Command::Tick(0.0),
    ];
    let before = workspace(vec![audio(1), audio(2)]).apply(GO);
    commands.into_iter().for_each(|command| {
        let after = before.clone().apply(command.clone());
        assert_eq!(shown(&after), shown(&before), "{command:?}");
        assert_eq!(running(&after), running(&before), "{command:?}");
        assert_eq!(standby(&after), standby(&before), "{command:?}");
        assert!(after.undoable().is_none(), "{command:?}");
    });
}
