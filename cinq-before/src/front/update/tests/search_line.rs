// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys of Vim's command line in the search line: recalling the
//! searches accepted earlier

use iced::keyboard::{Key, Modifiers, key::Named};

use super::{
    chord,
    search::{searched, show, typed},
};
use crate::{
    back::CueId,
    front::{App, Message, message::LineKey, update::Searched},
};

fn keyed(app: App, keys: &[LineKey]) -> App {
    keys.iter()
        .fold(app, |app, &key| app.handled(Message::SearchKey(key)).0)
}

fn text(app: &App) -> Option<&str> {
    app.search.as_ref().map(|search| search.text.as_str())
}

/// Accepts each of `words` as a search in turn, then opens the line again
fn after_searching(words: &[&str]) -> App {
    let open = |app: App| app.handled(Message::Search { backward: false }).0;
    let app = words
        .iter()
        .fold(show(Some(CueId(2))), |app, word| searched(open(app), word));
    open(app)
}

#[test]
fn keys_of_the_command_line_are_the_arrows_up_and_down() {
    let of = |key: Key, modifiers| LineKey::of(chord(key, modifiers));
    let (control, none) = (Modifiers::CTRL, Modifiers::empty());
    let letter = |text: &str| Key::Character(text.into());
    assert_eq!(of(Key::Named(Named::ArrowUp), none), Some(LineKey::Older));
    assert_eq!(of(Key::Named(Named::ArrowDown), none), Some(LineKey::Newer));
    assert_eq!(of(Key::Named(Named::ArrowUp), control), None);
    assert_eq!(of(letter("w"), control), None, "a text field takes it");
    assert_eq!(of(letter("u"), control), None, "a text field takes it");
}

#[test]
fn arrow_up_and_down_recall_the_searches_that_start_with_the_text_typed() {
    let line = || after_searching(&["wind", "rain", "storm", "thunder", "stor"]);
    // Without text every search is older, newest first
    assert_eq!(text(&keyed(line(), &[LineKey::Older])), Some("stor"));
    assert_eq!(text(&keyed(line(), &[LineKey::Older; 3])), Some("storm"));
    // With text only the searches that start with it
    let first = keyed(typed(line(), "st"), &[LineKey::Older]);
    assert_eq!(text(&first), Some("stor"));
    let second = keyed(first, &[LineKey::Older]);
    assert_eq!(text(&second), Some("storm"));
    // Nothing older that starts with it, so it stays
    let third = keyed(second, &[LineKey::Older]);
    assert_eq!(text(&third), Some("storm"));
    // Back through the newer, and then the text typed
    let back = keyed(third, &[LineKey::Newer]);
    assert_eq!(text(&back), Some("stor"));
    let typed_again = keyed(back, &[LineKey::Newer]);
    assert_eq!(text(&typed_again), Some("st"));
    let nothing_newer = keyed(typed_again, &[LineKey::Newer]);
    assert_eq!(text(&nothing_newer), Some("st"));
}

#[test]
fn typing_after_a_recall_starts_the_recall_afresh() {
    let line = keyed(after_searching(&["storm", "stor"]), &[LineKey::Older; 2]);
    assert_eq!(text(&line), Some("storm"));
    // Text typed then is what the recall matches next
    let line = keyed(typed(line, "sto"), &[LineKey::Older]);
    assert_eq!(text(&line), Some("stor"));
}

#[test]
fn history_keeps_each_search_once_the_newest_last_and_at_most_fifty() {
    let many = (0..60).fold(Searched::default(), |searched, number| {
        searched.accepting(format!("cue{number}"), false)
    });
    let history = &many.history;
    assert_eq!(history.len(), 50);
    assert_eq!(history.first().map(String::as_str), Some("cue10"));
    assert_eq!(history.last().map(String::as_str), Some("cue59"));
    // Search accepted again moves to the end, and is kept once
    let again = after_searching(&["wind", "rain", "wind"]);
    assert_eq!(again.searched.history, ["rain", "wind"]);
}

#[test]
fn the_keys_do_nothing_without_the_search_line() {
    let closed = show(Some(CueId(2)));
    let (after, task) = closed.handled(Message::SearchKey(LineKey::Older));
    assert!(after.search.is_none());
    assert_eq!(task.units(), 0);
}

#[test]
fn keys_that_change_the_text_move_the_cursor_to_the_end_as_vim_does() {
    let units = |app: App, message| app.handled(message).1.units();
    let line = || after_searching(&["storm"]);
    // Typing the same text takes one task less than the key, which also
    // moves the cursor of the field
    let plain = units(typed(line(), "st"), Message::Searching("sto".to_owned()));
    let recalled = units(typed(line(), "st"), Message::SearchKey(LineKey::Older));
    assert_eq!(recalled, plain + 1);
}

#[test]
fn recall_goes_past_an_older_search_that_equals_the_text_typed() {
    let line = typed(after_searching(&["storm", "st"]), "st");
    let older = keyed(line, &[LineKey::Older]);
    let past = keyed(older, &[LineKey::Older]);
    assert_eq!(text(&past), Some("storm"));
}
