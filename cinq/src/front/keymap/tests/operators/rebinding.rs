// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how a rebinding changes the operators and their motions

use super::*;

#[test]
fn operator_key_follows_a_rebinding_that_doubles_a_key() {
    let keymap = Keymap::default().with(Action::Cut, vec![keys("D D")]);
    let across = |text| typed_on(&keymap, text).across();
    assert_eq!(across("D j"), Some((Action::Cut, Action::Next, None)));
    assert_eq!(across("2 D G"), Some((Action::Cut, Action::Last, Some(2))));
    assert_eq!(across("d j"), None);
    // Key of Copy stays an operator
    assert_eq!(across("y k"), Some((Action::Copy, Action::Previous, None)));
}

#[test]
fn rebound_motion_carries_over_to_the_operators() {
    let keymap = Keymap::default().with(Action::Next, vec![keys("e")]);
    assert_eq!(
        typed_on(&keymap, "d e").across(),
        Some((Action::Cut, Action::Next, None))
    );
}

#[test]
fn binding_that_is_not_a_doubled_key_makes_no_operator() {
    let keymap = Keymap::default()
        .with(Action::Cut, vec![keys("Control+x"), keys("c v")])
        .with(Action::Copy, vec![keys("Control+c")]);
    let outcome = |text| typed_on(&keymap, text);
    assert_eq!(outcome("c j").across(), None);
    assert_eq!(outcome("y j").across(), None);
    assert_eq!(outcome("Control+x j").across(), None);
    assert_eq!(outcome("c v").action, Some((Action::Cut, None)));
}

#[test]
fn own_binding_of_the_keys_wins_over_the_operator() {
    let keymap = Keymap::default().with(Action::Standby, vec![keys("d j")]);
    let outcome = keymap.parse(&keys("d"), press("j"));
    assert_eq!(outcome.action, Some((Action::Standby, None)));
    assert_eq!(outcome.motion, None);
}

#[test]
fn a_keymap_of_single_keys_has_no_operator() {
    let keymap = declared("cut = \"Control+x\"\nnext = \"ArrowDown\"");
    let outcome = typed_on(&keymap, "Control+x ArrowDown");
    assert_eq!(outcome.across(), None);
    assert_eq!(outcome.action, Some((Action::Next, None)));
}

#[test]
fn box_leaves_out_a_motion_that_an_own_binding_takes() {
    let keymap = Keymap::default().with(Action::Standby, vec![keys("d j")]);
    let next = following(&keymap, "d");
    assert!(next.contains(&("j".to_owned(), Action::Standby, None)));
    assert!(!next.contains(&("j".to_owned(), Action::Next, Some(Action::Cut))));
    // Another key of the motion, and the same key after a count, still reach it
    assert!(next.contains(&("ArrowDown".to_owned(), Action::Next, Some(Action::Cut))));
    let counted = following(&keymap, "d 3");
    assert!(counted.contains(&("j".to_owned(), Action::Next, Some(Action::Cut))));
}
