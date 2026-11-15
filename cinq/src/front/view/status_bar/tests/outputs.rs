// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the hints that the outputs view shows

use super::{
    super::{notice, yielding},
    cancel::COLUMNS,
    fitted, hinted, opened,
};
use crate::{
    back::{Command, OutputId, Workspace},
    front::{
        App,
        keymap::{Action, Preset},
    },
};

fn in_outputs(preset: Preset) -> App {
    App {
        outputs_shown: true,
        ..opened(preset, false, false)
    }
}

/// Returns the hints that fit in `room` columns
fn shown(app: &App, room: usize) -> String {
    fitted(&hinted(app), room, yielding(app))
}

#[test]
fn the_outputs_hint_the_way_back_and_the_keys_to_add_and_to_rename_an_output() {
    assert_eq!(
        shown(&in_outputs(Preset::Vi), COLUMNS),
        "o: add output | i: rename | Control+o: back | Space: GO | Escape: Panic | F1: help"
    );
    // The cue list needs none of them, and has nowhere to go back to
    assert_eq!(
        shown(&opened(Preset::Vi, false, false), COLUMNS),
        "Space: GO | Escape: Panic | F1: help"
    );
}

#[test]
fn show_mode_locks_the_outputs_so_that_no_key_to_edit_them_is_hinted() {
    let locked = App {
        workspace: Workspace::default().apply(Command::ShowMode(true)),
        ..in_outputs(Preset::Vi)
    };
    assert_eq!(
        shown(&locked, COLUMNS),
        "Control+o: back | Space: GO | Escape: Panic | F1: help"
    );
}

#[test]
fn hints_of_the_outputs_give_way_with_the_way_back_lasting_longest_after_help() {
    let app = in_outputs(Preset::Vi);
    let in_turn: Vec<_> = [81, 65, 53, 41, 25, 7]
        .into_iter()
        .map(|room| shown(&app, room))
        .collect();
    assert_eq!(
        in_turn,
        [
            "i: rename | Control+o: back | Space: GO | Escape: Panic | F1: help",
            "Control+o: back | Space: GO | Escape: Panic | F1: help",
            "Control+o: back | Escape: Panic | F1: help",
            "Control+o: back | F1: help",
            "F1: help",
            "",
        ]
    );
}

#[test]
fn the_cue_list_keeps_its_order_of_giving_way() {
    let cues = yielding(&App::default());
    assert_eq!(cues[..2], [Action::Forward, Action::Back]);
    assert_ne!(cues, yielding(&in_outputs(Preset::Vi)));
}

#[test]
fn a_cut_output_stays_a_cut_in_the_notice_and_in_undo_and_redo() {
    let sub = OutputId(2);
    let show = Workspace::default().apply(Command::AddOutput { at: 1, count: 1 });
    let told = |workspace: Workspace| {
        notice(&App {
            workspace,
            ..App::default()
        })
    };
    let cut = show.clone().apply(Command::CutOutputs(vec![sub]));
    assert_eq!(
        told(cut.clone()).as_deref(),
        Some("Cut the output Output 1 | u: undo")
    );
    let undone = cut.clone().apply(Command::Undo);
    assert_eq!(
        told(undone.clone()).as_deref(),
        Some("Undid: cut the output Output 1 | Control+r: redo")
    );
    let redone = undone.apply(Command::Redo);
    assert_eq!(
        told(redone).as_deref(),
        Some("Redid: cut the output Output 1 | u: undo")
    );
    // A delete says so, as it did
    let deleted = show.apply(Command::DeleteOutputs(vec![sub]));
    assert_eq!(
        told(deleted).as_deref(),
        Some("Deleted the output Output 1 | u: undo")
    );
}
