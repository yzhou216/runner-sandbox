// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the hint of the key that cancels what is typed

use super::{
    super::order::yielding, YIELDING, fitted, has_hint, hinted, holding, opened, opened_with,
    taken, texts,
};
use crate::front::{
    App,
    app::Notices,
    keymap::{Action, Keymap, Press, desk},
};
use iced::keyboard::{Key, Modifiers};

/// Columns of a bar 1024 pixels wide in DejaVu Sans Mono at 16 pixels.
/// The test counts them rather than measuring the font, since the
/// monospace font that a build finds differs from one system to another.
pub(super) const COLUMNS: usize = 104;

fn cancel_hint(app: &App) -> Option<String> {
    texts(app)
        .into_iter()
        .find(|hint| hint.ends_with(": cancel"))
}

fn press(key: &str) -> Option<Press> {
    Press::from_key(&Key::Character(key.into()), Modifiers::empty())
}

#[test]
fn the_bar_hints_the_key_of_cancel_while_keys_wait() {
    let waiting = |keymap| App {
        keys: vec![press("d").expect("a key")],
        ..opened_with(keymap, false, false)
    };
    assert_eq!(cancel_hint(&opened(false, false)), None);
    assert_eq!(
        cancel_hint(&waiting(Keymap::default())).as_deref(),
        Some("Control+c: cancel")
    );
    // A keymap that binds no key to it has no hint
    assert_eq!(cancel_hint(&waiting(desk(""))), None);
}

#[test]
fn the_key_of_cancel_stays_in_a_bar_of_1024_pixels_while_go_and_panic_give_way() {
    let shown = |remark: &'static str| {
        let app = App {
            keys: vec![press("d").expect("a key")],
            notices: Notices {
                remark: Some(remark),
                ..Notices::default()
            },
            ..opened(false, false)
        };
        fitted(&hinted(&app), COLUMNS - taken(&app), &YIELDING)
    };
    // GO gives way first, then Panic, while cancel stays
    let short = shown("Copied cue 1 to the clipboard");
    assert!(short.contains("Control+c: cancel") && short.contains("Escape: Panic"));
    assert!(!short.contains("GO"));
    let long = shown("Searched for a cue whose name holds the words that were typed");
    assert!(long.contains("Control+c: cancel"));
    assert!(!long.contains("GO") && !long.contains("Panic"));
}

fn leaving(keymap: Keymap, held: bool) -> Vec<String> {
    let app = App {
        keyboard: holding(held),
        ..opened_with(keymap, false, false)
    };
    hinted(&app)
        .into_iter()
        .filter(|(action, _)| matches!(action, Some(Action::NextField | Action::Cancel)))
        .map(|(_, hint)| hint)
        .collect()
}

#[test]
fn the_bar_hints_how_to_leave_a_field_that_has_the_keyboard() {
    assert_eq!(leaving(Keymap::default(), false), Vec::<String>::new());
    assert_eq!(
        leaving(Keymap::default(), true),
        ["Tab: next field", "Control+c: leave"]
    );
    // A keymap that binds no key to cancel gets no new one
    assert_eq!(leaving(desk(""), true), ["Tab: next field"]);
}

#[test]
fn an_open_menu_and_the_help_page_take_the_place_of_those_hints() {
    let told = |menu, help| {
        let app = App {
            keyboard: holding(true).with_menu(menu),
            ..opened(menu, help)
        };
        texts(&app).iter().any(|hint| hint.contains("leave"))
    };
    assert!(told(false, false));
    assert!(!told(true, false), "a menu has its own keys");
    assert!(!told(false, true), "the page covers the field");
}

#[test]
fn the_hints_of_a_field_give_way_in_turn_with_go_and_panic_before_the_key_that_leaves() {
    let app = App {
        keyboard: holding(true),
        ..opened(false, false)
    };
    // No help hint here: a field or a list that has the keyboard keeps the
    // key of help from acting, and `?` types in a field
    let shown = |room| fitted(&hinted(&app), room, yielding(&app));
    let all = shown(COLUMNS);
    assert_eq!(
        all,
        "Enter: open | Tab: next field | Control+c: leave | Space: GO | Escape: Panic | f: hint"
    );
    // The key of the next field goes first, as the border shows the field,
    // then that of hint mode, and what Enter does stays with the rest in a bar
    // of 1024 pixels that has the name of a show and two statuses at its left
    let without_field = shown(all.len() - 1);
    assert_eq!(
        without_field,
        "Enter: open | Control+c: leave | Space: GO | Escape: Panic | f: hint"
    );
    assert_eq!(
        shown(without_field.len() - 1),
        "Enter: open | Control+c: leave | Space: GO | Escape: Panic"
    );
    assert_eq!(shown(58), shown(without_field.len() - 1));
    assert_eq!(shown(57), "Control+c: leave | Space: GO | Escape: Panic");
    // Then GO, whose button the transport shows, and Panic, while the key that
    // leaves the field outlasts both
    assert_eq!(shown(43), "Control+c: leave | Escape: Panic");
    assert_eq!(shown(31), "Control+c: leave");
}

#[test]
fn the_bar_shows_keys_typed_so_far_joined_without_the_space_that_is_go() {
    let app = App {
        keys: ["2", "d"].into_iter().filter_map(press).collect(),
        ..opened(false, false)
    };
    assert_eq!(hinted(&app)[0].1, "2d");
}

#[test]
fn the_search_line_of_a_keymap_with_no_key_to_cancel_names_the_key_that_leaves() {
    let hints = |text: &str| {
        let (mut app, _) = opened_with(desk(""), false, false).searching(false);
        if let Some(search) = app.search.as_mut() {
            search.text = text.to_owned();
        }
        texts(&app)
    };
    assert!(hints("").contains(&"Backspace: leave".to_owned()));
    assert!(hints("kno").contains(&"Enter: find".to_owned()));
    let (vi, _) = opened(false, false).searching(false);
    assert!(has_hint(&vi, "Control+c: cancel"));
}
