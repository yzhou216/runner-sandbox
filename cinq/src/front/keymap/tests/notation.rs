// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how keys are written, and of how the keys that iced reports
//! match what is written

use iced::keyboard::{Key, Modifiers, key::Named};

use super::*;

#[test]
fn notation_writes_keys_the_way_it_reads_them() {
    [
        ("Space Enter  Escape F35", "Space Enter Escape F35"),
        ("Control+d Alt+x Shift+Tab", "Control+d Alt+x Shift+Tab"),
        (
            "Alt+Control+ArrowUp Control++",
            "Control+Alt+ArrowUp Control++",
        ),
        ("Shift+a G < + Shift+Space", "A G < + Shift+Space"),
    ]
    .into_iter()
    .for_each(|(text, wanted)| assert_eq!(written(&keys(text)), wanted));
}

#[test]
fn notation_rejects_what_cannot_be_bound() {
    let accepted: Vec<_> = [
        "", "Foo", "escape", "gg", "Control+", "Shift", "Super", "Meta+x", "Shift+1",
    ]
    .into_iter()
    .filter(|text| binding(text).is_ok())
    .collect();
    assert!(accepted.is_empty(), "{accepted:?}");
}

#[test]
fn keys_from_iced_match_the_names_of_the_specification() {
    let none = Modifiers::empty();
    let named = |named: Named, modifiers| Press::from_key(&Key::Named(named), modifiers);
    assert_eq!(named(Named::ArrowDown, none), one("ArrowDown"));
    assert_eq!(named(Named::MediaPlayPause, none), one("MediaPlayPause"));
    assert_eq!(named(Named::Space, none), one("Space"));
    assert_eq!(named(Named::Space, Modifiers::SHIFT), one("Shift+Space"));
    assert_eq!(named(Named::Tab, Modifiers::SHIFT), one("Shift+Tab"));
    assert_eq!(
        Press::from_key(&Key::Character("G".into()), Modifiers::SHIFT),
        one("G")
    );
    assert_eq!(named(Named::Shift, Modifiers::SHIFT), None);
    assert_eq!(named(Named::Super, none), None);
}

#[test]
fn keys_that_type_characters_show_joined_and_others_keep_their_spaces() {
    [
        ("d d", "dd"),
        ("g T", "gT"),
        ("2 d 3 j", "2d3j"),
        ("j", "j"),
        ("z Enter", "z Enter"),
        ("g Control+x", "g Control+x"),
        ("Control+w w", "Control+w w"),
        ("Space", "Space"),
    ]
    .into_iter()
    .for_each(|(text, wanted)| assert_eq!(shown(&keys(text)), wanted));
}

#[test]
fn the_screen_joins_the_keys_of_a_sequence_and_the_file_keeps_their_spaces() {
    let keymap = Keymap::from(Preset::Vi);
    assert_eq!(keymap.listed(Action::NextTab), "gt");
    assert_eq!(keymap.key(Action::RowToMiddle).as_deref(), Some("zz"));
    assert_eq!(
        keymap.hint(Action::Copy, "copy").as_deref(),
        Some("yy: copy")
    );
    assert_eq!(keymap.listed(Action::RowToTop), "zt, z Enter");
    assert_eq!(written(&keys("g g")), "g g");
}
