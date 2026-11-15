// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the hint of the key that cancels what is typed

use iced::keyboard::{Key, Modifiers};
use unicode_width::UnicodeWidthStr;

use super::{super::status, fitted, hinted, opened};
use crate::front::{
    App,
    keymap::{Action, Preset, Press},
};

/// Columns of a bar 1024 pixels wide in DejaVu Sans Mono at 16 pixels.
/// The test counts them rather than measuring the font, since the
/// monospace font that a build finds differs from one system to another.
const COLUMNS: usize = 103;

/// Returns the hint of the key of cancel, if the bar shows one
fn cancel_hint(app: &App) -> Option<String> {
    hinted(app)
        .into_iter()
        .find_map(|(action, hint)| (action == Some(Action::Cancel)).then_some(hint))
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
        let taken: usize = status(&app).iter().map(|part| part.text.width()).sum();
        fitted(&hinted(&app), COLUMNS - taken)
    };
    // GO gives way first, then Panic, while cancel stays
    let short = shown("Copied cue 1");
    assert!(short.contains("Control+c: cancel") && short.contains("Escape: Panic"));
    assert!(!short.contains("GO"));
    let long = shown("Searched for a cue whose name holds the words that were typed");
    assert!(long.contains("Control+c: cancel"));
    assert!(!long.contains("GO") && !long.contains("Panic"));
}
