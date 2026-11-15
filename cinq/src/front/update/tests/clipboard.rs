// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of copying, cutting, and pasting cues

use super::*;
use crate::back::{Cue, Kind};

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
    // Undoing the first paste returns the selection to cue 6, which it was
    // made below, so that the second paste goes there as well
    assert_eq!(rows(&app)[4..8], [5, 6, 23, 7]);
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
fn show_mode_refuses_cutting_and_pasting_but_lets_a_cue_be_copied() {
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
    // Cut copied nothing, so that the clipboard still holds cue 4 for the
    // paste that follows, while copy replaces what it held
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
        open.workspace.cue(CueId(6)).map(|cue| cue.name.clone())
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

#[test]
fn a_clipboard_of_more_than_a_thousand_cues_pastes_a_thousand() {
    let cues = (1..=1500).map(audio).collect();
    let app = App {
        workspace: Workspace::new(vec![CueList::new("Main", cues)]),
        selected: Some(CueId(1)),
        ..App::default()
    };
    let app = after(app, [Message::Copy(1500), Message::Paste(Place::After, 1)]);
    assert_eq!(rows(&app).len(), 1500 + 1000);
    let app = after(app, [Message::Paste(Place::After, 2)]);
    assert_eq!(rows(&app).len(), 1500 + 2000);
}

/// Returns the state of a show of audio cue 1, fade cue 2 that targets it,
/// and audio cue 3, with cue 1 selected
pub(super) fn fading() -> App {
    let fade = Cue::new(CueId(2), "2", Kind::Fade, "Fade")
        .target(CueId(1))
        .build();
    App {
        workspace: Workspace::new(vec![CueList::new("Main", vec![audio(1), fade, audio(3)])]),
        selected: Some(CueId(1)),
        ..App::default()
    }
}

/// Returns the identity and target of each cue of the first list
pub(super) fn targets(app: &App) -> Vec<(u32, Option<u32>)> {
    app.workspace.lists()[0]
        .cues
        .iter()
        .map(|cue| (cue.id.0, cue.target.map(|target| target.0)))
        .collect()
}

#[test]
fn a_fade_cut_with_its_target_targets_the_new_copy_where_it_is_pasted() {
    let app = after(
        fading(),
        [
            Message::Cut(2),
            Message::Select(CueId(3)),
            Message::Paste(Place::After, 1),
        ],
    );
    assert_eq!(targets(&app), [(3, None), (4, None), (5, Some(4))]);
}

#[test]
fn a_fade_copied_with_its_target_targets_the_new_copy_and_the_old_one_stays() {
    let app = after(
        fading(),
        [
            Message::Copy(2),
            Message::Select(CueId(3)),
            Message::Paste(Place::After, 1),
        ],
    );
    let expected = [(1, None), (2, Some(1)), (3, None), (4, None), (5, Some(4))];
    assert_eq!(targets(&app), expected);
}

#[test]
fn each_paste_of_a_count_targets_the_copy_of_its_own_round() {
    let app = after(
        fading(),
        [
            Message::Copy(2),
            Message::Select(CueId(3)),
            Message::Paste(Place::After, 2),
        ],
    );
    assert_eq!(
        targets(&app)[3..],
        [(4, None), (5, Some(4)), (6, None), (7, Some(6))]
    );
}

#[test]
fn a_fade_copied_without_its_target_keeps_it_while_it_is_there() {
    let app = after(
        fading(),
        [
            Message::Select(CueId(2)),
            Message::Copy(1),
            Message::Paste(Place::After, 1),
        ],
    );
    assert_eq!(targets(&app)[2], (4, Some(1)));
}
