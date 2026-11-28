// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of what a copy or cut tells in the status bar

use iced::keyboard::Key;

use super::*;
use crate::front::keymap::Action;

#[test]
fn copy_and_cut_tell_what_went_to_the_clipboard() {
    let copied = after(show(Some(CueId(4))), [Message::Copy(3)]);
    assert_eq!(on_clipboard(&copied), Some(("Copied 3 cues", false)));
    let cut = after(show(Some(CueId(4))), [Message::Cut(1)]);
    assert_eq!(on_clipboard(&cut), Some(("Cut cue 4", true)));
    let nothing = after(show(None), [Message::Copy(3)]);
    assert_eq!(on_clipboard(&nothing), None);
}

#[test]
fn what_a_copy_told_stays_through_pending_keys_and_goes_with_the_next_action() {
    let copied = after(show(Some(CueId(4))), [Message::Copy(1)]);
    let (pending, _) = copied.next(Message::Typed {
        press: press(Key::Character("g".into())),
        repeat: false,
    });
    assert!(on_clipboard(&pending).is_some());
    let (moved, _) = pending.next(Message::Step(1));
    assert_eq!(on_clipboard(&moved), None);
}

#[test]
fn what_a_cut_told_stays_through_moves_and_list_switches_for_a_paste_into_the_other_list() {
    let cut = typing(show(Some(CueId(4))), "d d");
    let moved = typing(cut, "j k l");
    assert_eq!(on_clipboard(&moved), Some(("Cut cue 4", true)));
    assert_eq!(moved.list, 1);
    // Key that does anything else takes it away
    assert_eq!(on_clipboard(&typing(moved, "u")), None);
    let pasted = typing(show(Some(CueId(4))), "y y l p");
    assert_eq!(on_clipboard(&pasted), None);
    assert_eq!(pasted.workspace.lists()[1].cues.len(), 3);
}

#[test]
fn operators_over_a_motion_tell_the_cues_cut_or_copied_as_the_keys_of_a_cut_do() {
    let across = |operator, motion, count| {
        show(Some(CueId(4)))
            .handled(Message::Across(operator, motion, count))
            .0
    };
    assert_eq!(
        on_clipboard(&across(Action::Cut, Action::Next, Some(2))),
        Some(("Cut 3 cues", true))
    );
    assert_eq!(
        on_clipboard(&across(Action::Copy, Action::Last, None)),
        Some(("Copied 9 cues", false))
    );
    // Delete is not a cut, so it tells nothing of the clipboard
    assert_eq!(
        on_clipboard(&across(Action::Delete, Action::Next, None)),
        None
    );
}

#[test]
fn a_cut_is_told_as_a_cut_by_the_change_it_records_and_a_delete_as_a_delete() {
    let done = |message| after(show(Some(CueId(4))), [message]);
    assert_eq!(told(&done(Message::Cut(2))), "cut 2 cues");
    assert_eq!(told(&done(Message::Delete(2))), "deleted 2 cues");
    let operated = done(Message::Across(Action::Cut, Action::Next, None));
    assert_eq!(told(&operated), "cut 2 cues");
    // Undoing tells the same words
    let undone = done(Message::Cut(1)).handled(Message::Undo(1)).0;
    assert_eq!(told(&undone), "cut cue 4");
}

#[test]
fn an_operator_over_a_motion_that_finds_no_rows_leaves_what_a_copy_told() {
    let copied = typing(show(Some(CueId(1))), "y y");
    // Nothing lies above the first cue, nor below the last
    let up = typing(copied, "d k");
    assert_eq!(on_clipboard(&up), Some(("Copied cue 1", false)));
    let last = typing(up.handled(Message::Select(CueId(12))).0, "y j");
    assert_eq!(on_clipboard(&last), Some(("Copied cue 1", false)));
}
