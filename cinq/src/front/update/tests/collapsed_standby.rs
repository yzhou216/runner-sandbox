// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of switching to a list whose standby cue a collapsed group hides,
//! which selects the row of the group and leaves it collapsed

use super::{
    collapsed::{closed, collapsed, selected},
    *,
};
use crate::back::Command;

/// Returns the show of `collapsed` with a second list in front, and standby
/// of the list `collapsed` makes on `standby`
fn behind(standby: u32) -> App {
    let app = collapsed(1);
    let lists = [CueList::new("Front", Vec::new())]
        .into_iter()
        .chain(app.workspace.lists().iter().cloned())
        .collect();
    App {
        workspace: Workspace::new(lists),
        list: 0,
        selected: None,
        ..app
    }
    .send(Command::Standby {
        list: 1,
        cue: Some(CueId(standby)),
    })
}

#[test]
fn switching_to_a_list_selects_the_group_that_hides_standby() {
    let app = after(behind(5), [Message::SwitchList(1)]);
    assert_eq!(app.list, 1);
    assert_eq!(selected(&app), Some(2));
    assert!(closed(&app, 2));
    assert_eq!(app.row(), Some(1));
}

#[test]
fn the_list_keys_select_the_group_that_hides_standby() {
    let app = after(behind(3), [Message::StepList(1)]);
    assert_eq!(selected(&app), Some(2));
    assert!(closed(&app, 2));
}

#[test]
fn switching_to_a_list_selects_a_shown_standby_cue_as_before() {
    let app = after(behind(7), [Message::SwitchList(1)]);
    assert_eq!(selected(&app), Some(7));
    let first = after(behind(2), [Message::SwitchList(1)]);
    assert_eq!(selected(&first), Some(2));
}
