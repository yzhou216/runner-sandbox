// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the hints that the outputs view shows

use super::{
    super::{hints::notices::notice, order::yielding},
    cancel::COLUMNS,
    hints_fitted, opened, opened_with,
};
use crate::{
    back::{Command, OutputId, Workspace},
    front::{
        App,
        keymap::{Action, Keymap, desk},
        update::Show,
    },
};

fn in_outputs(keymap: Keymap) -> App {
    App {
        outputs_shown: true,
        ..opened_with(keymap, false, false)
    }
}

/// Returns the state with a file for the show, which names no key to open one
fn on_file() -> App {
    App {
        show: Show::in_file("/shows/storm.cinq.toml"),
        ..opened(false, false)
    }
}

#[test]
fn the_outputs_hint_the_way_back_and_the_keys_to_add_and_to_rename_an_output() {
    assert_eq!(
        hints_fitted(&in_outputs(Keymap::default()), COLUMNS),
        "o: add output | i: rename | Control+o: back to Main | Space: GO | Escape: Panic | ?: help | f: hint"
    );
    // Cue list needs none of them, and has nowhere to go back to
    assert_eq!(
        hints_fitted(&on_file(), COLUMNS),
        "Space: GO | Escape: Panic | ?: help | gt: next tab | l: next list | f: hint"
    );
}

#[test]
fn show_mode_locks_the_outputs_so_that_no_key_to_edit_them_is_hinted() {
    let locked = App {
        workspace: Workspace::default().apply(Command::ShowMode(true)),
        ..in_outputs(Keymap::default())
    };
    assert_eq!(
        hints_fitted(&locked, COLUMNS),
        "Control+o: back to Main | Space: GO | Escape: Panic | ?: help | f: hint"
    );
}

#[test]
fn hints_of_the_outputs_give_way_with_the_way_back_lasting_longest_after_help() {
    let app = in_outputs(Keymap::default());
    let in_turn: Vec<_> = [81, 65, 53, 41, 25, 6]
        .into_iter()
        .map(|room| hints_fitted(&app, room))
        .collect();
    assert_eq!(
        in_turn,
        [
            "i: rename | Control+o: back to Main | Space: GO | Escape: Panic | ?: help",
            "Control+o: back to Main | Space: GO | Escape: Panic | ?: help",
            "Control+o: back to Main | Escape: Panic | ?: help",
            "Control+o: back to Main | ?: help",
            "?: help",
            "",
        ]
    );
}

#[test]
fn the_cue_list_keeps_its_order_of_giving_way() {
    let cues = yielding(&App::default());
    assert_eq!(
        cues[..13],
        [
            Action::NextList,
            Action::NextTab,
            Action::Hint,
            Action::Open,
            Action::Standby,
            Action::Forward,
            Action::Back,
            Action::Go,
            Action::Panic,
            Action::Fullscreen,
            Action::Save,
            Action::Help,
            Action::ChangeName
        ]
    );
    assert_ne!(cues, yielding(&in_outputs(Keymap::default())));
}

#[test]
fn a_cut_output_stays_a_cut_in_the_notice_and_in_undo_and_redo() {
    let sub = OutputId(2);
    let show = Workspace::default().apply(Command::add_output(1, 1));
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
    let undone = cut.apply(Command::Undo);
    assert_eq!(
        told(undone.clone()).as_deref(),
        Some("Undid: cut the output Output 1 | Control+r: redo")
    );
    let redone = undone.apply(Command::Redo);
    assert_eq!(
        told(redone).as_deref(),
        Some("Redid: cut the output Output 1 | u: undo")
    );
    // Delete says so, as it did
    let deleted = show.apply(Command::DeleteOutputs(vec![sub]));
    assert_eq!(
        told(deleted).as_deref(),
        Some("Deleted the output Output 1 | u: undo")
    );
}

#[test]
fn the_outputs_hint_the_key_that_changes_the_name_to_rename_in_a_keymap_with_no_cancel() {
    let keymap = desk("edit = []\nchange_name = \"F2\"\nadd_below = \"Control+a\"");
    let told = |room| hints_fitted(&in_outputs(keymap.clone()), room);
    assert!(told(COLUMNS).contains("F2: rename"));
    assert!(
        told(COLUMNS).find("Control+a: add output") < told(COLUMNS).find("F2: rename"),
        "the order of the hints stays"
    );
    // Gives way before add output, and so before GO and Panic
    assert!(!told(70).contains("F2: rename") && told(70).contains("Escape: Panic"));
    assert!(
        (1..COLUMNS).all(|room| !told(room).contains("F2: rename")
            || told(room).contains("Control+a: add output")),
        "add output shows at every width where rename does"
    );
    assert!(
        (1..COLUMNS).any(|room| told(room).contains("Control+a: add output")
            && !told(room).contains("F2: rename")),
        "rename gives way first"
    );
}
