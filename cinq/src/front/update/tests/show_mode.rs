// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of show mode, which locks the cues against every message that edits
//! them

use super::{operators::rows, *};
use crate::front::keymap::Action::{Cut, Delete, Indent, Next};

/// Returns the state in show mode after cue 4 is copied and deleted and the
/// cue that took its place is indented, and then undone when `undone`, so that
/// both undo and redo have a change to act on
fn locked_after_edits(undone: bool) -> App {
    let edited = after(
        show(Some(CueId(4))),
        [Message::Copy(1), Message::Delete(1), Message::Indent(1)],
    );
    locked(if undone {
        after(edited, [Message::Undo(1)])
    } else {
        edited
    })
}

/// Returns what the messages that edit would change: the cues of the list, their
/// depths, the standby cue, the selection, the clipboard, and the latest change
fn state(app: &App) -> String {
    let list = &app.workspace.lists()[0];
    format!(
        "{:?}",
        (
            rows(app),
            list.depths(),
            list.standby,
            app.selected,
            held(app),
            app.workspace.latest_change(),
        )
    )
}

#[test]
fn show_mode_refuses_every_message_that_edits_the_cues() {
    let messages = [
        Message::Add(Place::After, 1),
        Message::Add(Place::After, 3),
        Message::Add(Place::Before, 2),
        Message::Move(1),
        Message::Move(-1),
        Message::Indent(1),
        Message::Outdent(1),
        Message::Cut(2),
        Message::Paste(Place::After, 1),
        Message::Paste(Place::Before, 2),
        Message::Delete(2),
        Message::Undo(1),
        Message::Redo(1),
        Message::Across(Cut, Next, None),
        Message::Across(Delete, Next, None),
        Message::Across(Cut, Next, Some(2)),
        Message::Across(Delete, Next, Some(2)),
        Message::Across(Indent, Next, None),
    ];
    [false, true].into_iter().for_each(|undone| {
        let before = state(&locked_after_edits(undone));
        messages.iter().for_each(|message| {
            let app = after(locked_after_edits(undone), [message.clone()]);
            assert_eq!(state(&app), before, "{message:?}, undone {undone}");
        });
    });
}
