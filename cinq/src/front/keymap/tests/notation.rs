// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how keys are written, and of how iced reports them

use iced::keyboard::{self, Key, Modifiers, key::Named};

use super::*;
use crate::front::keymap::KeyPress;

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
    .for_each(|(text, wanted)| assert_eq!(shown(&keys(text)), wanted, "{text}"));
}

#[test]
fn the_screen_joins_the_keys_of_a_sequence_and_the_file_keeps_their_spaces() {
    let keymap = Keymap::default();
    assert_eq!(keymap.listed(Action::NextTab), "gt");
    assert_eq!(keymap.key(Action::RowToMiddle).as_deref(), Some("zz"));
    assert_eq!(
        keymap.hint(Action::Copy, "copy").as_deref(),
        Some("yy: copy")
    );
    assert_eq!(keymap.listed(Action::RowToTop), "zt, z Enter");
    assert_eq!(written(&keys("g g")), "g g");
}

#[test]
fn caps_lock_leaves_the_letter_of_a_key_held_with_control_or_alt_to_shift() {
    // Caps Lock makes the capital that xkb reports for Control+s, so that
    // without Shift the key is still the lowercase one
    let chord = |text: &str, modifiers| Press::from_key(&Key::Character(text.into()), modifiers);
    let (control_shift, alt_shift) = (
        Modifiers::CTRL | Modifiers::SHIFT,
        Modifiers::ALT | Modifiers::SHIFT,
    );
    assert_eq!(chord("S", Modifiers::CTRL), one("Control+s"));
    assert_eq!(chord("J", Modifiers::ALT), one("Alt+j"));
    assert_eq!(chord("s", control_shift), one("Control+S"));
    assert_eq!(chord("S", control_shift), one("Control+S"));
    assert_eq!(chord("j", alt_shift), one("Alt+J"));
    assert_eq!(chord("G", Modifiers::empty()), one("G"));
    assert_eq!(chord("/", Modifiers::CTRL), one("Control+/"));
}

#[test]
fn alt_with_a_letter_is_that_letter_where_the_system_makes_another_character_of_it() {
    // Option+j arrives on Darwin as the character that it types, `∆`
    let event = |key: &str, typed: &str, modifiers| keyboard::Event::KeyPressed {
        key: Key::Character(key.into()),
        modified_key: Key::Character(typed.into()),
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers,
        text: None,
        repeat: false,
    };
    let pressed = |event| KeyPress::of(&event).and_then(|key| key.press);
    assert_eq!(pressed(event("j", "∆", Modifiers::ALT)), one("Alt+j"));
    assert_eq!(
        pressed(event("j", "Ô", Modifiers::ALT | Modifiers::SHIFT)),
        one("Alt+J")
    );
    assert_eq!(pressed(event("j", "j", Modifiers::ALT)), one("Alt+j"));
    assert_eq!(pressed(event("j", "∆", Modifiers::empty())), one("∆"));
    assert_ne!(
        pressed(event("a", "ą", Modifiers::CTRL | Modifiers::ALT)),
        one("Control+Alt+a")
    );
}

#[test]
fn notation_reads_a_key_with_a_modifier_repeated_thousands_of_times() {
    let text = format!("{}a", "Control+".repeat(200_000));
    assert_eq!(keys(&text), keys("Control+a"));
}
