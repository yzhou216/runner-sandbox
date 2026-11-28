// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of adding, deleting, and pasting several outputs as one change, and
//! of a cut output taking back its identity

use super::{
    outputs::{delete_outputs, edit_output, names, rename_output, send_on, sends, sends_of_cue},
    *,
};
use crate::back::{Channels, Output, OutputEdit, OutputId};

fn ids(workspace: &Workspace) -> Vec<u32> {
    workspace
        .outputs()
        .iter()
        .map(|output| output.id.0)
        .collect()
}

/// Returns a show with House, Sub, and Lobby in that order, to which a cue
/// sends at -3 dB on Sub and the new cues send on House and Sub only
fn house_sub_and_lobby() -> Workspace {
    workspace(vec![audio(1)])
        .apply(rename_output(1, "House"))
        .apply(Command::add_output(1, 2))
        .apply(rename_output(2, "Sub"))
        .apply(edit_output(2, OutputEdit::NewCues(Some(-6.0))))
        .apply(rename_output(3, "Lobby"))
        .apply(send_on(&[(2, -3.0)]))
}

fn cut(show: &Workspace, ids: &[u32]) -> Vec<Output> {
    ids.iter()
        .map(|&id| {
            show.output(OutputId(id))
                .expect("the output is there")
                .clone()
        })
        .collect()
}

fn paste_outputs(show: Workspace, at: usize, outputs: Vec<Output>, restore: bool) -> Workspace {
    show.apply(Command::PasteOutputs {
        at,
        outputs,
        restore,
        anchor: None,
    })
}

/// Returns the show after the outputs `ids` were cut and pasted back
/// at `at`, as the register of outputs does
fn cut_and_pasted_back(show: Workspace, ids: &[u32], at: usize) -> Workspace {
    let taken = cut(&show, ids);
    paste_outputs(show.apply(delete_outputs(ids)), at, taken, true)
}

#[test]
fn a_counted_add_puts_the_outputs_in_a_row_as_one_change() {
    let added = workspace(Vec::new()).apply(Command::add_output(1, 3));
    assert_eq!(names(&added), ["Main", "Output 1", "Output 2", "Output 3"]);
    assert_eq!(ids(&added), [1, 2, 3, 4]);
    let undone = added.apply(Command::Undo);
    assert_eq!(names(&undone), ["Main"]);
}

#[test]
fn deleting_several_outputs_is_one_change_and_never_takes_every_output() {
    let show = house_sub_and_lobby();
    let two = show.clone().apply(delete_outputs(&[1, 2]));
    assert_eq!(names(&two), ["Lobby"]);
    assert_eq!(names(&two.apply(Command::Undo)), ["House", "Sub", "Lobby"]);
    let all = show.clone().apply(delete_outputs(&[1, 2, 3]));
    assert_eq!(names(&all), ["House", "Sub", "Lobby"]);
    let none = show.apply(delete_outputs(&[99]));
    assert_eq!(names(&none), ["House", "Sub", "Lobby"]);
    assert!(
        none.undoable()
            .is_some_and(|change| !matches!(change, crate::back::Change::Outputs { .. }))
    );
}

#[test]
fn a_cut_output_pasted_back_takes_back_its_identity_and_the_cues_that_sent_to_it() {
    let show = house_sub_and_lobby();
    let gone = show.clone().apply(delete_outputs(&[2]));
    assert_eq!(sends_of_cue(&gone), sends(&[]));
    let back = cut_and_pasted_back(show, &[2], 1);
    assert_eq!(names(&back), ["House", "Sub", "Lobby"]);
    assert_eq!(ids(&back), [1, 2, 3]);
    assert_eq!(sends_of_cue(&back), sends(&[(2, -3.0)]));
    assert_eq!(
        back.output(OutputId(2)).and_then(|output| output.new_cues),
        Some(-6.0)
    );
}

#[test]
fn a_cut_output_pasted_at_the_top_is_the_first_output_again() {
    let back = cut_and_pasted_back(house_sub_and_lobby(), &[2], 0);
    assert_eq!(names(&back), ["Sub", "House", "Lobby"]);
    assert_eq!(ids(&back), [2, 1, 3]);
}

#[test]
fn a_copy_gets_an_identity_a_free_name_and_no_new_cues_or_sends() {
    let show = house_sub_and_lobby();
    let copies = cut(&show, &[2]);
    let pasted = paste_outputs(show, 2, copies, false);
    assert_eq!(names(&pasted), ["House", "Sub", "Sub 2", "Lobby"]);
    assert_eq!(ids(&pasted), [1, 2, 4, 3]);
    let copy = pasted.output(OutputId(4)).expect("the copy is there");
    assert_eq!((copy.new_cues, copy.channels), (None, Channels::MAIN));
    assert_eq!(sends_of_cue(&pasted), sends(&[(2, -3.0)]));
}

#[test]
fn pasting_a_cut_output_twice_restores_it_once_and_copies_it_after() {
    let show = house_sub_and_lobby();
    let taken = cut(&show, &[2]);
    let pasted = paste_outputs(
        show.apply(delete_outputs(&[2])),
        1,
        [taken.clone(), taken].concat(),
        true,
    );
    assert_eq!(names(&pasted), ["House", "Sub", "Sub 2", "Lobby"]);
    assert_eq!(ids(&pasted), [1, 2, 4, 3]);
}

#[test]
fn a_restored_output_whose_name_is_taken_is_numbered_and_one_that_is_free_is_not() {
    let show = house_sub_and_lobby();
    let taken = cut(&show, &[2]);
    let gone = show
        .apply(delete_outputs(&[2]))
        .apply(rename_output(3, "Sub"));
    let back = paste_outputs(gone, 1, taken, true);
    assert_eq!(names(&back), ["House", "Sub 2", "Sub"]);
    assert_eq!(ids(&back), [1, 2, 3]);
}

#[test]
fn an_output_pasted_from_another_show_with_an_identity_here_is_a_copy() {
    let show = house_sub_and_lobby();
    let alien = cut(&show, &[1]);
    // The register of the front end would not ask to restore, since the show
    // is another one, but even so no identity is taken that is in use
    let pasted = paste_outputs(show, 3, alien, true);
    assert_eq!(names(&pasted), ["House", "Sub", "Lobby", "House 2"]);
    assert_eq!(ids(&pasted), [1, 2, 3, 4]);
}

#[test]
fn show_mode_locks_adding_deleting_and_pasting_outputs() {
    let show = house_sub_and_lobby().apply(Command::ShowMode(true));
    let taken = cut(&show, &[2]);
    let same = |show: Workspace| names(&show).join(",");
    let before = same(show.clone());
    assert_eq!(same(show.clone().apply(Command::add_output(0, 2))), before);
    assert_eq!(same(show.clone().apply(delete_outputs(&[1]))), before);
    assert_eq!(same(paste_outputs(show, 0, taken, false)), before);
}

#[test]
fn a_copy_of_a_numbered_name_takes_the_next_number_and_not_a_second_one() {
    let show = house_sub_and_lobby().apply(rename_output(2, "House R 2"));
    let copies = cut(&show, &[2]);
    let paste = |show: Workspace, outputs| paste_outputs(show, usize::MAX, outputs, false);
    // Each paste, of the one output or of two of it, takes the lowest free
    let one = paste(show.clone(), copies.clone());
    assert_eq!(names(&one), ["House", "House R 2", "Lobby", "House R 3"]);
    let again = paste(one, copies.clone());
    assert_eq!(names(&again)[3..], ["House R 3", "House R 4"]);
    let two = paste(show, [copies.clone(), copies].concat());
    assert_eq!(names(&two)[3..], ["House R 3", "House R 4"]);
    // Number that is the whole name stays, with the number added after it
    let bare = house_sub_and_lobby().apply(rename_output(2, "7"));
    let pasted = paste(bare.clone(), cut(&bare, &[2]));
    assert_eq!(names(&pasted)[3], "7 2");
}
