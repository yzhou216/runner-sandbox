// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that edit the show, which the stage window leaves to
//! the main window

use super::{focusing, opened};
use crate::{
    back::CueId,
    front::{App, update::tests::typing},
};

/// Returns the state of the interface after the window that `main` picks
/// takes the keyboard, with the only cue selected
fn focused(main: bool) -> App {
    let open = App {
        selected: Some(CueId(1)),
        ..opened()
    };
    focusing(open, !main)
}

/// Returns the number of cues of the first list and whether anything went
/// to the clipboard
fn found(app: &App) -> (usize, bool) {
    (
        app.workspace.lists()[0].cues.len(),
        app.notices.clipped.is_some(),
    )
}

#[test]
fn keys_that_edit_the_show_do_nothing_while_the_stage_window_has_the_keyboard() {
    [
        "o", "O", "x", "Delete", "Y", "p", "P", "d d", "y y", "d j", "y G",
    ]
    .into_iter()
    .for_each(|keys| {
        assert_eq!(found(&typing(focused(false), keys)), (1, false), "{keys}");
    });
}

#[test]
fn the_same_keys_act_from_the_main_window() {
    let from_main = |keys| found(&typing(focused(true), keys));
    assert_eq!(from_main("o"), (2, false));
    assert_eq!(from_main("Y"), (1, true));
    assert_eq!(from_main("x"), (0, true));
    assert_eq!(from_main("d d"), (0, true));
}

#[test]
fn undo_and_redo_wait_for_the_main_window_too() {
    let added = typing(focused(true), "o");
    let undone = typing(focusing(added, true), "u");
    assert_eq!(found(&undone).0, 2);
    let undone = typing(focusing(undone, false), "u");
    assert_eq!(found(&undone).0, 1);
    let redone = typing(focusing(undone, true), "Control+r");
    assert_eq!(found(&redone).0, 1);
}

#[test]
fn a_refused_key_tells_why_and_a_key_that_edits_nothing_tells_nothing() {
    let refused = typing(focused(false), "o");
    assert_eq!(
        refused.notices.remark,
        Some("Click the main window to edit")
    );
    assert_eq!(typing(focused(false), "j").notices.remark, None);
    assert_eq!(typing(focused(true), "o").notices.remark, None);
    // Next key that acts takes the remark back
    assert_eq!(typing(refused, "j").notices.remark, None);
}
