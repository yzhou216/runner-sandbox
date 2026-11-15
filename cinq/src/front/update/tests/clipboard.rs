// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of copying, cutting, and pasting cues

use super::*;

/// Returns the identities of the cues of the first list in order
fn rows(app: &App) -> Vec<u32> {
    app.workspace.lists()[0]
        .cues
        .iter()
        .map(|cue| cue.id.0)
        .collect()
}

fn after(app: App, messages: impl IntoIterator<Item = Message>) -> App {
    messages
        .into_iter()
        .fold(app, |app, message| app.handled(message).0)
}

#[test]
fn copied_cue_pastes_below_the_selection_and_becomes_the_selection() {
    let app = after(
        show(Some(CueId(4))),
        [Message::Copy(1), Message::Paste(Place::After, 1)],
    );
    assert_eq!(rows(&app)[..6], [1, 2, 3, 4, 22, 5]);
    assert_eq!(app.selected, Some(CueId(22)));
    let (copy, original) = (app.workspace.cue(CueId(22)), app.workspace.cue(CueId(4)));
    let settings =
        |cue: Option<&crate::back::Cue>| cue.map(|cue| (cue.name.clone(), cue.file.clone()));
    assert_eq!(settings(copy), settings(original));
}

#[test]
fn copy_with_a_count_takes_the_cues_below_and_paste_above_keeps_their_order() {
    let app = after(
        show(Some(CueId(3))),
        [
            Message::Copy(2),
            Message::Select(CueId(8)),
            Message::Paste(Place::Before, 1),
        ],
    );
    assert_eq!(rows(&app)[5..11], [6, 7, 22, 23, 8, 9]);
    assert_eq!(app.selected, Some(CueId(22)));
}

#[test]
fn paste_with_a_count_pastes_that_many_times_in_one_change() {
    let app = after(
        show(Some(CueId(4))),
        [Message::Copy(2), Message::Paste(Place::After, 3)],
    );
    assert_eq!(rows(&app).len(), 12 + 6);
    let undone = after(app, [Message::Undo(1)]);
    assert_eq!(rows(&undone).len(), 12);
}

#[test]
fn cut_then_paste_moves_a_cue_and_keeps_its_number() {
    let app = after(show(Some(CueId(2))), [Message::Cut(1)]);
    assert_eq!(rows(&app)[..3], [1, 3, 4]);
    assert_eq!(app.selected, Some(CueId(3)));
    let moved = after(
        app,
        [Message::Select(CueId(5)), Message::Paste(Place::After, 1)],
    );
    assert_eq!(rows(&moved)[..6], [1, 3, 4, 5, 22, 6]);
    assert_eq!(moved.workspace.cue(CueId(22)).unwrap().number, "2");
}

#[test]
fn delete_leaves_the_clipboard_alone_and_undo_does_not_touch_it() {
    let app = after(
        show(Some(CueId(4))),
        [
            Message::Copy(1),
            Message::Select(CueId(6)),
            Message::Delete(1),
            Message::Undo(1),
            Message::Paste(Place::After, 1),
            Message::Undo(1),
            Message::Paste(Place::After, 1),
        ],
    );
    // Undoing the first paste leaves the selection on the cue that took the
    // place of the pasted one, below which the second paste goes
    assert_eq!(rows(&app)[4..8], [5, 6, 7, 23]);
}

#[test]
fn pasting_with_an_empty_clipboard_does_nothing_and_tells_nothing() {
    let app = after(show(Some(CueId(4))), [Message::Paste(Place::After, 1)]);
    assert_eq!(rows(&app).len(), 12);
    assert_eq!(app.selected, Some(CueId(4)));
    assert!(app.workspace.latest_change().is_none());
    assert_eq!(app.told, None);
}

#[test]
fn copying_nothing_leaves_the_clipboard_as_it_was() {
    let app = after(
        show(Some(CueId(4))),
        [Message::Copy(1), Message::SwitchList(1)],
    );
    let app = App {
        selected: None,
        ..app
    };
    let app = after(app, [Message::Copy(1), Message::Paste(Place::After, 1)]);
    assert_eq!(app.workspace.lists()[1].cues.len(), 3);
}

#[test]
fn show_mode_refuses_copying_cutting_and_pasting() {
    let copied = after(show(Some(CueId(4))), [Message::Copy(1)]);
    let locked = after(
        copied,
        [
            Message::ShowMode(true),
            Message::Select(CueId(6)),
            Message::Cut(1),
            Message::Paste(Place::After, 1),
        ],
    );
    assert_eq!(rows(&locked).len(), 12);
    // Copy was refused as well, so that the clipboard still holds cue 4
    let open = after(
        locked,
        [
            Message::Copy(1),
            Message::ShowMode(false),
            Message::Paste(Place::After, 1),
        ],
    );
    assert_eq!(
        open.workspace.cue(CueId(22)).map(|cue| cue.name.clone()),
        open.workspace.cue(CueId(4)).map(|cue| cue.name.clone())
    );
    assert_eq!(rows(&open).len(), 13);
}

#[test]
fn paste_adds_at_most_a_thousand_cues() {
    let app = after(
        show(Some(CueId(4))),
        [Message::Copy(1), Message::Paste(Place::After, usize::MAX)],
    );
    assert_eq!(rows(&app).len(), 12 + 1000);
}
