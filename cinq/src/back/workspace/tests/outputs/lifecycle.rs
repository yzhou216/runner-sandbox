// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the outputs that a show starts with, adds, edits, and deletes, and
//! of the sends that new cues get

use super::*;

#[test]
fn show_starts_with_main_where_cues_that_name_no_outputs_send() {
    let show = workspace(vec![audio(1)]);
    assert_eq!(names(&show), ["Main"]);
    assert_eq!(sends_of_cue(&show), sends(&[(1, 0.0)]));
}

#[test]
fn outputs_add_after_one_another_under_names_of_their_own_and_keep_one() {
    let added = workspace(Vec::new())
        .apply(Command::add_output(usize::MAX, 1))
        .apply(Command::add_output(1, 1));
    assert_eq!(names(&added), ["Main", "Output 2", "Output 1"]);
    let renamed = added
        .apply(rename_output(2, "Main"))
        .apply(rename_output(3, " "));
    assert_eq!(names(&renamed), ["Main", "Output 2", "Output 1"]);
    let moved = renamed.apply(Command::MoveOutput {
        output: MAIN,
        by: 5,
    });
    assert_eq!(names(&moved), ["Output 2", "Output 1", "Main"]);
    let one = [MAIN, OutputId(2), OutputId(3)]
        .into_iter()
        .fold(moved, |show, id| {
            show.apply(Command::DeleteOutputs(vec![id]))
        });
    assert_eq!(names(&one), ["Output 2"]);
}

#[test]
fn new_cues_send_where_the_outputs_say_and_a_name_typed_out_undoes_at_once() {
    let show = house_and_sub().apply(add(0, Kind::Audio, Place::End));
    let new = show.cues().last().expect("the cue is added");
    assert_eq!(show.sends(new), sends(&[(1, 0.0), (2, -6.0)]));
    let typed = "Lobby"
        .char_indices()
        .map(|(end, _)| &"Lobby"[..=end])
        .fold(show, |show, name| show.apply(rename_output(2, name)));
    assert_eq!(names(&typed), ["House", "Lobby"]);
    let undone = typed.apply(Command::Undo);
    assert_eq!(names(&undone), ["House", "Sub"]);
}

#[test]
fn every_cue_of_a_counted_add_sends_where_the_outputs_say() {
    let show = house_and_sub().apply(add_row(Kind::Audio, Place::End, 3));
    let added: Vec<_> = show.cues().skip(1).map(|cue| show.sends(cue)).collect();
    assert_eq!(added, vec![sends(&[(1, 0.0), (2, -6.0)]); 3]);
}

#[test]
fn deleting_an_output_silences_its_sends_until_undo_brings_it_back() {
    let show = house_and_sub().apply(send_on(&[(1, 0.0), (2, -3.0)]));
    let deleted = show.apply(delete_outputs(&[2]));
    assert_eq!(sends_of_cue(&deleted), sends(&[(1, 0.0)]));
    let undone = deleted.apply(Command::Undo);
    assert_eq!(sends_of_cue(&undone), sends(&[(1, 0.0), (2, -3.0)]));
}

#[test]
fn output_edit_that_changes_nothing_or_move_past_the_end_leaves_redo_alone() {
    let undone = house_and_sub()
        .apply(rename_output(1, "Front"))
        .apply(Command::Undo);
    assert!(undone.redoable().is_some());
    let moved_past_the_end = Command::MoveOutput {
        output: OutputId(2),
        by: 1,
    };
    [rename_output(1, "House"), moved_past_the_end]
        .into_iter()
        .for_each(|command| {
            let after = undone.clone().apply(command.clone());
            assert!(after.redoable().is_some(), "{command:?}");
        });
}
