// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the hint of the key that cancels what is typed

use iced::keyboard::{Key, Modifiers};
use unicode_width::UnicodeWidthStr;

use super::{super::status, YIELDING, fitted, hinted, opened};
use crate::{
    back::{Cue, CueId, CueList, Kind, Workspace},
    front::{
        App,
        keymap::{Action, Preset, Press},
    },
};

/// Columns of a bar 1024 pixels wide in DejaVu Sans Mono at 16 pixels.
/// The test counts them rather than measuring the font, since the
/// monospace font that a build finds differs from one system to another.
pub(super) const COLUMNS: usize = 103;

/// Returns the hint of the key of cancel, if the bar shows one
fn cancel_hint(app: &App) -> Option<String> {
    hinted(app)
        .into_iter()
        .map(|(_, hint)| hint)
        .find(|hint| hint.ends_with(": cancel"))
}

#[test]
fn the_bar_hints_the_key_of_cancel_while_keys_wait() {
    let d = Press::from_key(&Key::Character("d".into()), Modifiers::empty()).expect("a key");
    let waiting = |preset| App {
        keys: vec![d],
        ..opened(preset, false, false)
    };
    assert_eq!(cancel_hint(&opened(Preset::Vi, false, false)), None);
    assert_eq!(
        cancel_hint(&waiting(Preset::Vi)).as_deref(),
        Some("Control+c: cancel")
    );
    // Cua binds no key to it
    let cua = waiting(Preset::Cua);
    assert_eq!(cancel_hint(&cua), None);
}

#[test]
fn the_key_of_cancel_stays_in_a_bar_of_1024_pixels_while_go_and_panic_give_way() {
    let d = Press::from_key(&Key::Character("d".into()), Modifiers::empty()).expect("a key");
    let shown = |remark: &'static str| {
        let app = App {
            keys: vec![d],
            remark: Some(remark),
            ..opened(Preset::Vi, false, false)
        };
        let taken: usize = status(&app, app.show.name().into_owned())
            .iter()
            .map(|part| part.text.width())
            .sum();
        fitted(&hinted(&app), COLUMNS - taken, &YIELDING)
    };
    // GO gives way first, then Panic, while cancel stays
    let short = shown("Copied cue 1");
    assert!(short.contains("Control+c: cancel") && short.contains("Escape: Panic"));
    assert!(!short.contains("GO"));
    let long = shown("Searched for a cue whose name holds the words that were typed");
    assert!(long.contains("Control+c: cancel"));
    assert!(!long.contains("GO") && !long.contains("Panic"));
}

/// Returns the hints that tell how to leave a field
fn leaving(preset: Preset, held: bool) -> Vec<String> {
    let app = App {
        held,
        ..opened(preset, false, false)
    };
    hinted(&app)
        .into_iter()
        .filter(|(action, _)| matches!(action, Some(Action::NextField | Action::Cancel)))
        .map(|(_, hint)| hint)
        .collect()
}

#[test]
fn the_bar_hints_how_to_leave_a_field_that_has_the_keyboard() {
    assert_eq!(leaving(Preset::Vi, false), Vec::<String>::new());
    assert_eq!(
        leaving(Preset::Vi, true),
        ["Tab: next field", "Control+c: leave"]
    );
    // Cua binds no key to cancel, and gets no new one
    assert_eq!(leaving(Preset::Cua, true), ["Tab: next field"]);
}

#[test]
fn an_open_menu_and_the_help_page_take_the_place_of_those_hints() {
    let told = |menu, help| {
        let app = App {
            held: true,
            ..opened(Preset::Vi, menu, help)
        };
        hinted(&app).iter().any(|(_, hint)| hint.contains("leave"))
    };
    assert!(told(false, false));
    assert!(!told(true, false), "a menu has its own keys");
    assert!(!told(false, true), "the page covers the field");
}

#[test]
fn the_hints_of_leaving_a_field_give_way_after_the_help_page() {
    let app = App {
        held: true,
        ..opened(Preset::Vi, false, false)
    };
    let all = fitted(&hinted(&app), COLUMNS, &YIELDING);
    assert_eq!(
        all,
        "Tab: next field | Control+c: leave | Space: GO | Escape: Panic | F1: help"
    );
    let narrow = fitted(&hinted(&app), 45, &YIELDING);
    assert!(narrow.contains("Control+c: leave"), "{narrow}");
    assert!(
        !narrow.contains("GO") && !narrow.contains("Panic"),
        "{narrow}"
    );
}

#[test]
fn the_bar_shows_keys_typed_so_far_joined_without_the_space_that_is_go() {
    let press = |key: &str| Press::from_key(&Key::Character(key.into()), Modifiers::empty());
    let app = App {
        keys: ["2", "d"].iter().filter_map(|key| press(key)).collect(),
        ..opened(Preset::Vi, false, false)
    };
    assert_eq!(hinted(&app)[0].1, "2d");
}

#[test]
fn the_bar_hints_enter_while_the_empty_file_field_has_the_keyboard() {
    let cue = |file: &str| {
        Cue::new(CueId(1), "1", Kind::Audio, "Rain")
            .file(file)
            .build()
    };
    let told = |file: &str, file_held: bool| {
        let app = App {
            held: true,
            file_held,
            selected: Some(CueId(1)),
            workspace: Workspace::new(vec![CueList::new("Main", vec![cue(file)])]),
            ..opened(Preset::Vi, false, false)
        };
        hinted(&app)
            .iter()
            .any(|(_, hint)| hint == "Enter: choose a file")
    };
    assert!(told("", true));
    assert!(!told("rain.flac", true), "a named file needs no chooser");
    assert!(!told("", false), "another field has the keyboard");
}
