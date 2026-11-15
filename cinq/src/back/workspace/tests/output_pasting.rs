// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of adding, deleting, and pasting several outputs as one change, and
//! of a cut output taking back its identity

use super::*;
use crate::back::{Channels, Output, OutputEdit, OutputId, Sends};

fn names(workspace: &Workspace) -> Vec<&str> {
    workspace
        .outputs()
        .iter()
        .map(|output| output.name.as_str())
        .collect()
}

fn ids(workspace: &Workspace) -> Vec<u32> {
    workspace
        .outputs()
        .iter()
        .map(|output| output.id.0)
        .collect()
}

fn sends(pairs: &[(u32, f32)]) -> Sends {
    pairs
        .iter()
        .map(|&(id, level)| (OutputId(id), level))
        .collect()
}

fn edit_output(id: u32, edit: OutputEdit) -> Command {
    Command::EditOutput(OutputId(id), edit)
}

/// Returns a show with House, Sub, and Lobby in that order, to which a cue
/// sends at -3 dB on Sub and the new cues send on House and Sub only
fn house_sub_and_lobby() -> Workspace {
    workspace(vec![audio(1)])
        .apply(edit_output(1, OutputEdit::Name("House".to_owned())))
        .apply(Command::AddOutput { at: 1, count: 2 })
        .apply(edit_output(2, OutputEdit::Name("Sub".to_owned())))
        .apply(edit_output(2, OutputEdit::NewCues(Some(-6.0))))
        .apply(edit_output(3, OutputEdit::Name("Lobby".to_owned())))
        .apply(Command::Edit(
            CueId(1),
            Edit::Sends(Some(sends(&[(2, -3.0)]))),
        ))
}

fn deleted(show: Workspace, ids: &[u32]) -> Workspace {
    show.apply(Command::DeleteOutputs(
        ids.iter().copied().map(OutputId).collect(),
    ))
}

#[test]
fn a_counted_add_puts_the_outputs_in_a_row_as_one_change() {
    let added = workspace(Vec::new()).apply(Command::AddOutput { at: 1, count: 3 });
    assert_eq!(names(&added), ["Main", "Output 1", "Output 2", "Output 3"]);
    assert_eq!(ids(&added), [1, 2, 3, 4]);
    let undone = added.apply(Command::Undo);
    assert_eq!(names(&undone), ["Main"]);
}

#[test]
fn deleting_several_outputs_is_one_change_and_never_takes_every_output() {
    let show = house_sub_and_lobby();
    let two = deleted(show.clone(), &[1, 2]);
    assert_eq!(names(&two), ["Lobby"]);
    assert_eq!(names(&two.apply(Command::Undo)), ["House", "Sub", "Lobby"]);
    let all = deleted(show.clone(), &[1, 2, 3]);
    assert_eq!(names(&all), ["House", "Sub", "Lobby"]);
    let none = deleted(show, &[99]);
    assert_eq!(names(&none), ["House", "Sub", "Lobby"]);
    assert!(
        none.undoable()
            .is_some_and(|change| !matches!(change, crate::back::Change::Outputs { .. }))
    );
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

#[test]
fn a_cut_output_pasted_back_takes_back_its_identity_and_the_cues_that_sent_to_it() {
    let show = house_sub_and_lobby();
    let taken = cut(&show, &[2]);
    let gone = deleted(show, &[2]);
    let cue = |show: &Workspace| show.cue(CueId(1)).cloned().expect("the cue is there");
    assert_eq!(gone.sends(&cue(&gone)), sends(&[]));
    let back = gone.apply(Command::PasteOutputs {
        at: 1,
        outputs: taken,
        restore: true,
    });
    assert_eq!(names(&back), ["House", "Sub", "Lobby"]);
    assert_eq!(ids(&back), [1, 2, 3]);
    assert_eq!(back.sends(&cue(&back)), sends(&[(2, -3.0)]));
    assert_eq!(
        back.output(OutputId(2)).and_then(|output| output.new_cues),
        Some(-6.0)
    );
}

#[test]
fn a_cut_output_pasted_at_the_top_is_the_first_output_again() {
    let show = house_sub_and_lobby();
    let taken = cut(&show, &[2]);
    let back = deleted(show, &[2]).apply(Command::PasteOutputs {
        at: 0,
        outputs: taken,
        restore: true,
    });
    assert_eq!(names(&back), ["Sub", "House", "Lobby"]);
    assert_eq!(ids(&back), [2, 1, 3]);
}

#[test]
fn a_copy_gets_an_identity_a_free_name_and_no_new_cues_or_sends() {
    let show = house_sub_and_lobby();
    let copies = cut(&show, &[2]);
    let pasted = show.apply(Command::PasteOutputs {
        at: 2,
        outputs: copies,
        restore: false,
    });
    assert_eq!(names(&pasted), ["House", "Sub", "Sub 2", "Lobby"]);
    assert_eq!(ids(&pasted), [1, 2, 4, 3]);
    let copy = pasted.output(OutputId(4)).expect("the copy is there");
    assert_eq!((copy.new_cues, copy.channels), (None, Channels::MAIN));
    let cue = pasted.cue(CueId(1)).expect("the cue is there");
    assert_eq!(pasted.sends(cue), sends(&[(2, -3.0)]));
}

#[test]
fn pasting_a_cut_output_twice_restores_it_once_and_copies_it_after() {
    let show = house_sub_and_lobby();
    let taken = cut(&show, &[2]);
    let pasted = deleted(show, &[2]).apply(Command::PasteOutputs {
        at: 1,
        outputs: [taken.clone(), taken].concat(),
        restore: true,
    });
    assert_eq!(names(&pasted), ["House", "Sub", "Sub 2", "Lobby"]);
    assert_eq!(ids(&pasted), [1, 2, 4, 3]);
}

#[test]
fn a_restored_output_whose_name_is_taken_is_numbered_and_one_that_is_free_is_not() {
    let show = house_sub_and_lobby();
    let taken = cut(&show, &[2]);
    let gone = deleted(show, &[2]).apply(edit_output(3, OutputEdit::Name("Sub".to_owned())));
    let back = gone.apply(Command::PasteOutputs {
        at: 1,
        outputs: taken,
        restore: true,
    });
    assert_eq!(names(&back), ["House", "Sub 2", "Sub"]);
    assert_eq!(ids(&back), [1, 2, 3]);
}

#[test]
fn an_output_pasted_from_another_show_with_an_identity_here_is_a_copy() {
    let show = house_sub_and_lobby();
    let alien = cut(&show, &[1]);
    // The register of the front end would not ask to restore, since the show
    // is another one, but even so no identity is taken that is in use
    let pasted = show.apply(Command::PasteOutputs {
        at: 3,
        outputs: alien,
        restore: true,
    });
    assert_eq!(names(&pasted), ["House", "Sub", "Lobby", "House 2"]);
    assert_eq!(ids(&pasted), [1, 2, 3, 4]);
}

#[test]
fn show_mode_locks_adding_deleting_and_pasting_outputs() {
    let show = house_sub_and_lobby().apply(Command::ShowMode(true));
    let taken = cut(&show, &[2]);
    let same = |show: &Workspace| names(show).join(",");
    let before = same(&show);
    assert_eq!(
        same(&show.clone().apply(Command::AddOutput { at: 0, count: 2 })),
        before
    );
    assert_eq!(same(&deleted(show.clone(), &[1])), before);
    let pasted = show.apply(Command::PasteOutputs {
        at: 0,
        outputs: taken,
        restore: false,
    });
    assert_eq!(same(&pasted), before);
}
