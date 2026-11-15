// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys of Vim's command line in the search line: deleting the
//! word before the cursor and everything before it, and recalling the
//! searches accepted earlier

use iced::keyboard::{Key, Modifiers, key::Named};

use super::{
    chord,
    search::{opened, searched, show, typed},
};
use crate::front::{
    App, Message,
    message::LineKey,
    update::{Searched, search::line::without_last_word},
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
    let app = words
        .iter()
        .fold(show(Some(crate::back::CueId(2))), |app, word| {
            searched(app.handled(Message::Search { backward: false }).0, word)
        });
    app.handled(Message::Search { backward: false }).0
}

#[test]
fn keys_of_the_command_line_are_control_w_control_u_and_the_arrows_up_and_down() {
    let of = |key: Key, modifiers| LineKey::of(chord(key, modifiers));
    let (control, none) = (Modifiers::CTRL, Modifiers::empty());
    let letter = |text: &str| Key::Character(text.into());
    assert_eq!(of(letter("w"), control), Some(LineKey::DeleteWord));
    assert_eq!(of(letter("u"), control), Some(LineKey::DeleteAll));
    assert_eq!(of(Key::Named(Named::ArrowUp), none), Some(LineKey::Older));
    assert_eq!(of(Key::Named(Named::ArrowDown), none), Some(LineKey::Newer));
    assert_eq!(of(letter("w"), none), None);
    assert_eq!(of(Key::Named(Named::ArrowUp), control), None);
    assert_eq!(of(letter("n"), control), None);
}

#[test]
fn control_w_deletes_the_word_before_the_cursor_as_vim_does() {
    assert_eq!(without_last_word("storm opening"), "storm ");
    assert_eq!(without_last_word("storm "), "");
    assert_eq!(without_last_word("storm  "), "");
    assert_eq!(without_last_word("a/b"), "a/");
    assert_eq!(without_last_word("a.."), "a");
    assert_eq!(without_last_word("snake_case"), "");
    assert_eq!(without_last_word(""), "");
    assert_eq!(without_last_word("tonnerre éclair"), "tonnerre ");
}

#[test]
fn control_w_and_control_u_edit_the_text_and_the_selection_follows() {
    let line = typed(opened(1), "storm op");
    assert_eq!(text(&line), Some("storm op"));
    let word = keyed(line, &[LineKey::DeleteWord]);
    assert_eq!(text(&word), Some("storm "));
    let all = keyed(word, &[LineKey::DeleteAll]);
    assert_eq!(text(&all), Some(""));
    // Nothing matches no text, so the selection is back where it began
    assert_eq!(all.selected, Some(crate::back::CueId(1)));
    // On an empty line they change nothing, and close nothing
    let empty = keyed(all, &[LineKey::DeleteWord, LineKey::DeleteAll]);
    assert_eq!(text(&empty), Some(""));
}

#[test]
fn arrow_up_and_down_recall_the_searches_that_start_with_the_text_typed() {
    let line = after_searching(&["wind", "rain", "storm", "thunder", "stor"]);
    // Without text every search is older, newest first
    let up = |app, times: usize| keyed(app, &vec![LineKey::Older; times]);
    assert_eq!(text(&up(line, 1)), Some("stor"));
    let line = after_searching(&["wind", "rain", "storm", "thunder", "stor"]);
    assert_eq!(text(&up(line, 3)), Some("storm"));
    // With text only the searches that start with it
    let line = typed(
        after_searching(&["wind", "rain", "storm", "thunder", "stor"]),
        "st",
    );
    let first = up(line, 1);
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
    let line = after_searching(&["storm", "stor"]);
    let line = keyed(line, &[LineKey::Older, LineKey::Older]);
    assert_eq!(text(&line), Some("storm"));
    // Text typed then is what the recall matches next
    let line = typed(line, "sto");
    let line = keyed(line, &[LineKey::Older]);
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
    let closed = show(Some(crate::back::CueId(2)));
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
    let word = units(typed(line(), "st"), Message::SearchKey(LineKey::DeleteWord));
    let all = units(typed(line(), "st"), Message::SearchKey(LineKey::DeleteAll));
    assert_eq!((word, all), (recalled, recalled));
}
