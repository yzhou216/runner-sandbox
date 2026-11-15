// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of adding cues, one or as many as a count types

use super::*;

/// Returns the identities of the cues of the first list in order
fn rows(app: &App) -> Vec<u32> {
    app.workspace.lists()[0]
        .cues
        .iter()
        .map(|cue| cue.id.0)
        .collect()
}

fn typing(app: App, keys: &[&str]) -> App {
    keys.iter().fold(app, |app, &key| {
        app.typed(press(Key::Character(key.into())), false).0
    })
}

#[test]
fn one_cue_added_below_is_selected_as_before() {
    let app = typing(show(Some(CueId(4))), &["o"]);
    assert_eq!(rows(&app)[3..6], [4, 22, 5]);
    assert_eq!(app.selected, Some(CueId(22)));
}

#[test]
fn count_adds_that_many_cues_below_in_one_change_and_selects_the_last() {
    let app = typing(show(Some(CueId(4))), &["3", "o"]);
    assert_eq!(rows(&app)[3..8], [4, 22, 23, 24, 5]);
    // Vim leaves the cursor on the last line that `3o` inserts
    assert_eq!(app.selected, Some(CueId(24)));
    let undone = typing(app, &["u"]);
    assert_eq!(rows(&undone).len(), 12);
    // Back on the cue that was selected, as `o u` is in Vim
    assert_eq!(undone.selected, Some(CueId(4)));
}

#[test]
fn count_adds_that_many_cues_above_and_selects_the_last_above_the_selection() {
    let app = typing(show(Some(CueId(4))), &["2", "O"]);
    assert_eq!(rows(&app)[2..7], [3, 22, 23, 4, 5]);
    assert_eq!(app.selected, Some(CueId(23)));
}

#[test]
fn undoing_a_count_above_takes_all_its_cues_and_selects_the_cue_they_were_above() {
    let app = typing(show(Some(CueId(4))), &["3", "O"]);
    assert_eq!(rows(&app)[2..7], [3, 22, 23, 24, 4]);
    let undone = typing(app, &["u"]);
    assert_eq!(rows(&undone), (1..=12).collect::<Vec<_>>());
    assert_eq!(undone.selected, Some(CueId(4)));
}

#[test]
fn count_without_a_selection_adds_at_the_end() {
    let app = typing(show(None), &["2", "o"]);
    assert_eq!(rows(&app)[10..], [11, 12, 22, 23]);
    assert_eq!(app.selected, Some(CueId(23)));
}

#[test]
fn add_stops_at_a_thousand_cues() {
    let (app, _) = show(Some(CueId(4))).handled(Message::Add(Place::After, usize::MAX));
    assert_eq!(rows(&app).len(), 12 + 1000);
}

#[test]
fn show_mode_refuses_adding_with_a_count() {
    let (locked, _) = show(Some(CueId(4))).handled(Message::ShowMode(true));
    let app = typing(locked, &["3", "o"]);
    assert_eq!(rows(&app).len(), 12);
    assert_eq!(app.selected, Some(CueId(4)));
}

#[test]
fn y_copies_the_selected_cue_and_takes_a_count() {
    let app = typing(show(Some(CueId(4))), &["Y", "p"]);
    assert_eq!(rows(&app)[3..6], [4, 22, 5]);
    let app = typing(show(Some(CueId(4))), &["2", "Y", "p"]);
    assert_eq!(rows(&app)[3..7], [4, 22, 23, 5]);
}
