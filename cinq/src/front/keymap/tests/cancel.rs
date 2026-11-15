// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the key that cancels what is typed, as Control+c does in Vim

use super::*;

#[test]
fn control_c_cancels_in_the_vi_keymap_and_acts_in_fields() {
    let keymap = Keymap::default();
    assert_eq!(action("Control+c"), Some((Action::Cancel, None)));
    let press = one("Control+c").expect("a key");
    assert!(keymap.acts_in_fields(press));
    assert!(keymap.cancels(press));
    assert_eq!(keymap.in_menu(press), Some(Action::Cancel));
    assert!(!keymap.acts_at_once(press));
    assert_eq!(keymap.conflicts(), Vec::<String>::new());
}

#[test]
fn control_c_stays_the_copy_of_the_cua_keymap_which_binds_nothing_to_cancel() {
    let keymap = Keymap::from(Preset::Cua);
    assert!(keymap.keys(Action::Cancel).is_empty());
    assert!(!keymap.cancels(one("Control+c").expect("a key")));
    assert_eq!(keymap.conflicts(), Vec::<String>::new());
}

#[test]
fn cancel_drops_the_keys_typed_so_far_as_any_complete_command_does() {
    assert_eq!(typed("d Control+c").pending, []);
    assert_eq!(action("3 Control+c"), Some((Action::Cancel, Some(3))));
    assert_eq!(typed("g Control+c").pending, []);
}

#[test]
fn a_field_that_captured_a_key_lets_only_a_key_that_types_no_character_cancel() {
    let keymap = Keymap::default().with(Action::Cancel, vec![keys("Control+c"), keys("q")]);
    let press = |text| one(text).expect("a key");
    // Control+c copies a selection in a field, which cancels all the same
    assert!(keymap.acts_when_captured(press("Control+c"), true));
    // A letter that a user binds to cancel types in the field
    assert!(!keymap.acts_when_captured(press("q"), true));
    // A named key types nothing
    let named = Keymap::default().with(Action::Cancel, vec![keys("F5")]);
    assert!(named.acts_when_captured(press("F5"), false));
    // GO and Panic keep their rule: a character key never acts when captured
    assert!(keymap.acts_when_captured(press("Space"), false));
    assert!(!keymap.acts_when_captured(press("Space"), true));
}
