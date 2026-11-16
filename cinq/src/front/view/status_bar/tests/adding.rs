// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the hint that follows a cue just added

use super::{hinted, opened};
use crate::{
    back::{Command, Kind, Place, Workspace},
    front::{App, keymap::Preset},
};

/// Returns the first hint, which tells the latest change, after `command`
fn told(preset: Preset, command: Command) -> Option<String> {
    let workspace = Workspace::default();
    let app = App {
        workspace: workspace.apply(command),
        ..opened(preset, false, false)
    };
    hinted(&app).into_iter().next().map(|(_, hint)| hint)
}

fn add() -> Command {
    Command::Add {
        list: 0,
        kind: Kind::Audio,
        at: Place::End,
        count: 1,
    }
}

#[test]
fn a_cue_just_added_hints_the_key_that_renames_it_before_the_key_of_undo() {
    assert_eq!(
        told(Preset::Vi, add()).as_deref(),
        Some("Added cue 1 | cc: rename | u: undo")
    );
    assert_eq!(
        told(Preset::Cua, add()).as_deref(),
        Some("Added cue 1 | F2: rename | Control+z: undo")
    );
}

#[test]
fn the_show_mode_that_locks_the_name_hints_no_rename() {
    let locked = Workspace::default()
        .apply(add())
        .apply(Command::ShowMode(true));
    let app = App {
        workspace: locked,
        ..opened(Preset::Vi, false, false)
    };
    assert!(
        hinted(&app)
            .iter()
            .all(|(_, hint)| !hint.contains("rename"))
    );
}
