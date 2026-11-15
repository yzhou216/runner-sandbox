// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of what a copy or cut tells in the status bar

use iced::keyboard::Key;

use super::*;

fn after(app: App, messages: impl IntoIterator<Item = Message>) -> App {
    messages
        .into_iter()
        .fold(app, |app, message| app.handled(message).0)
}

fn told(app: &App) -> Option<(&str, bool)> {
    app.clipped
        .as_ref()
        .map(|clipped| (clipped.told.as_str(), clipped.cut))
}

#[test]
fn copy_and_cut_tell_what_went_to_the_clipboard() {
    let copied = after(show(Some(CueId(4))), [Message::Copy(3)]);
    assert_eq!(told(&copied), Some(("Copied 3 cues", false)));
    let cut = after(show(Some(CueId(4))), [Message::Cut(1)]);
    assert_eq!(told(&cut), Some(("Cut cue 4", true)));
    let nothing = after(show(None), [Message::Copy(3)]);
    assert_eq!(told(&nothing), None);
}

#[test]
fn what_a_copy_told_stays_through_pending_keys_and_goes_with_the_next_action() {
    let copied = after(show(Some(CueId(4))), [Message::Copy(1)]);
    let (pending, _) = copied.next(Message::Typed {
        press: super::press(Key::Character("g".into())),
        repeat: false,
    });
    assert!(told(&pending).is_some());
    let (moved, _) = pending.next(Message::Step(1));
    assert_eq!(told(&moved), None);
}

#[test]
fn operators_over_a_motion_tell_the_cues_cut_or_copied_as_the_keys_of_a_cut_do() {
    let across = |operator, motion, count| {
        show(Some(CueId(4)))
            .handled(Message::Across(operator, motion, count))
            .0
    };
    use crate::front::keymap::Action::{Copy, Cut, Delete, Last, Next};
    assert_eq!(
        told(&across(Cut, Next, Some(2))),
        Some(("Cut 3 cues", true))
    );
    assert_eq!(
        told(&across(Copy, Last, None)),
        Some(("Copied 9 cues", false))
    );
    // Delete is not a cut, so it tells nothing of the clipboard
    assert_eq!(told(&across(Delete, Next, None)), None);
}

#[test]
fn a_cut_is_told_as_a_cut_by_the_change_it_records_and_a_delete_as_a_delete() {
    use crate::front::{keymap::Action, view::format};
    let done = |message| after(show(Some(CueId(4))), [message]);
    let told = |app: &App| {
        let (_, change) = app
            .workspace
            .latest_change()
            .expect("a removal is a change");
        format::change(&app.workspace, change)
    };
    assert_eq!(told(&done(Message::Cut(2))), "cut 2 cues");
    assert_eq!(told(&done(Message::Delete(2))), "deleted 2 cues");
    let operated = done(Message::Across(Action::Cut, Action::Next, None));
    assert_eq!(told(&operated), "cut 2 cues");
    // Undoing tells the same words
    let undone = done(Message::Cut(1)).handled(Message::Undo(1)).0;
    assert_eq!(told(&undone), "cut cue 4");
}
