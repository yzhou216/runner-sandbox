// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of reading keys typed in a row, and of the keys that act alone

use super::*;
use crate::front::Message;

#[test]
fn counts_repeat_motions() {
    assert_eq!(action("j"), Some((Action::Next, None)));
    assert_eq!(action("1 2 j"), Some((Action::Next, Some(12))));
    assert_eq!(action("3 ArrowUp"), Some((Action::Previous, Some(3))));
    assert_eq!(action("Control+n"), Some((Action::Next, None)));
    assert_eq!(action("2 Control+p"), Some((Action::Previous, Some(2))));
}

#[test]
fn keys_act_alone_when_bound_by_themselves_or_to_go_and_panic() {
    let keymap = Keymap::default();
    let alone = |text: &str| one(text).and_then(|press| keymap.alone(press));
    assert_eq!(alone("Control+n"), Some(Action::Next));
    assert_eq!(alone("Shift+Escape"), Some(Action::Panic));
    assert_eq!(alone("g"), None);
    assert_eq!(alone("z"), None);
}

#[test]
fn enter_acts_at_once_only_when_bound_to_go_or_panic() {
    let enter = one("Enter").expect("Enter is a named key");
    assert!(!Keymap::default().acts_at_once(enter));
    let bound = Keymap::default().with(Action::Go, vec![keys("Enter")]);
    assert!(bound.acts_at_once(enter));
}

#[test]
fn commands_of_two_keys_wait_for_the_second() {
    assert_eq!(typed("2 d").pending, keys("2 d"));
    assert_eq!(action("g g"), Some((Action::First, None)));
    assert_eq!(action("7 G"), Some((Action::Last, Some(7))));
    assert_eq!(action("g T"), Some((Action::PreviousTab, None)));
}

#[test]
fn go_and_panic_act_at_once_with_any_modifiers() {
    let outcome = typed("3 d Space");
    assert_eq!(outcome.action, Some((Action::Go, None)));
    assert!(outcome.pending.is_empty());
    assert_eq!(action("g Control+Escape"), Some((Action::Panic, None)));
}

#[test]
fn key_bound_with_the_modifiers_held_goes_to_its_own_action() {
    let keymap = Keymap::default().with(Action::Panic, vec![keys("Shift+Space")]);
    let alone = |text: &str| one(text).and_then(|press| keymap.alone(press));
    assert_eq!(alone("Shift+Space"), Some(Action::Panic));
    assert_eq!(alone("Control+Space"), Some(Action::Go));
    assert_eq!(alone("Space"), Some(Action::Go));
}

#[test]
fn keys_typed_halfway_list_what_can_follow_them() {
    let keymap = Keymap::default();
    let next = |text: &str| following(&keymap, text);
    let g = [
        ("j".to_owned(), Action::Next, None),
        ("k".to_owned(), Action::Previous, None),
        ("g".to_owned(), Action::First, None),
        ("t".to_owned(), Action::NextTab, None),
        ("T".to_owned(), Action::PreviousTab, None),
    ];
    assert_eq!(next("g"), g);
    assert_eq!(next("1 2 g"), g);
    assert_eq!(next("2 d")[0], ("d".to_owned(), Action::Cut, None));
    assert!(next("3").is_empty());
    assert_eq!(keymap.continuations(&[]).count(), 0);
}

#[test]
fn keys_of_vim_cut_help_first_and_last_take_counts_as_their_twins_do() {
    assert_actions(&[
        ("x", Action::Cut, None),
        ("3 x", Action::Cut, Some(3)),
        ("2 Delete", Action::Cut, Some(2)),
        ("F1", Action::Help, None),
        ("?", Action::Help, None),
        ("4 Control+Home", Action::First, Some(4)),
        ("Control+End", Action::Last, None),
        ("7 Control+End", Action::Last, Some(7)),
    ]);
    // Cutting with x leaves no second key to wait for
    let keymap = Keymap::default();
    assert!(typed("x").pending.is_empty());
    assert!(keymap.continuations(&keys("x")).next().is_none());
    assert_eq!(keymap.alone(press("x")), Some(Action::Cut));
}

#[test]
fn keys_of_delete_cut_and_help_stay_with_a_field_that_has_the_keyboard() {
    [Action::Delete, Action::Cut, Action::Help]
        .into_iter()
        .for_each(|action| assert!(!action.acts_in_fields(), "{action:?}"));
}

#[test]
fn counts_of_vi_keys_reach_undo_redo_and_the_tabs() {
    assert_actions(&[
        ("3 u", Action::Undo, Some(3)),
        ("2 Control+r", Action::Redo, Some(2)),
        ("2 g t", Action::NextTab, Some(2)),
        ("2 g T", Action::PreviousTab, Some(2)),
    ]);
    assert!(matches!(Action::Undo.message(Some(3)), Message::Undo(3)));
    assert!(matches!(Action::Redo.message(None), Message::Redo(1)));
    assert!(matches!(
        Action::NextTab.message(Some(2)),
        Message::ShowTab(2)
    ));
    assert!(matches!(Action::NextTab.message(None), Message::StepTab(1)));
    assert!(matches!(
        Action::PreviousTab.message(Some(2)),
        Message::StepTab(-2)
    ));
}

#[test]
fn keys_that_move_by_a_page_are_those_of_vim_and_act_in_no_field() {
    let keymap = Keymap::default();
    [
        ("Control+d", Action::HalfPageDown),
        ("Control+u", Action::HalfPageUp),
        ("Control+f", Action::PageDown),
        ("PageDown", Action::PageDown),
        ("Control+b", Action::PageUp),
        ("PageUp", Action::PageUp),
    ]
    .into_iter()
    .for_each(|(keys, page)| {
        assert_actions(&[(keys, page, None), (&format!("4 {keys}"), page, Some(4))]);
        assert!(!keymap.acts_in_fields(press(keys)), "{keys}");
    });
    // Keys of the prefixes g and d stay theirs, apart from Control+d
    assert_actions(&[("g g", Action::First, None), ("d d", Action::Cut, None)]);
    assert_eq!(typed("d").pending.len(), 1);
}

#[test]
fn vi_copies_with_y_y_or_capital_y_cuts_with_d_d_and_pastes_with_p_and_capital_p() {
    assert_actions(&[
        ("y y", Action::Copy, None),
        ("3 y y", Action::Copy, Some(3)),
        ("Y", Action::Copy, None),
        ("3 Y", Action::Copy, Some(3)),
        // Y is bound by itself, so it makes no operator
        ("Y j", Action::Next, None),
        ("d d", Action::Cut, None),
        ("p", Action::PasteBelow, None),
        ("2 P", Action::PasteAbove, Some(2)),
    ]);
    assert!(typed("Y").pending.is_empty());
    // Cutting is what deleting does in Vim, so that no key only deletes
    assert!(Keymap::default().keys(Action::Delete).is_empty());
}

#[test]
fn counts_reach_the_actions_that_add_cues() {
    assert_actions(&[
        ("3 o", Action::AddBelow, Some(3)),
        ("2 O", Action::AddAbove, Some(2)),
    ]);
    assert!(matches!(
        Action::AddBelow.message(Some(3)),
        Message::Add(_, 3)
    ));
    assert!(matches!(Action::AddAbove.message(None), Message::Add(_, 1)));
}

#[test]
fn undo_redo_hints_and_adding_a_cue_act_in_a_closed_list_and_not_in_a_text_field() {
    let vi = Keymap::default();
    ["u", "Control+r", "f", "o", "O"]
        .into_iter()
        .for_each(|key| {
            assert!(
                vi.acts_in_lists(press(key)) && !vi.acts_in_fields(press(key)),
                "{key}"
            );
        });
    // Keys of scrolling and of the other edits stay out of the list
    assert!(!vi.acts_in_lists(press("Control+e")) && !vi.acts_in_lists(press("j")));
    // Keys that act in every field act in a list too
    assert!(vi.acts_in_lists(press("Tab")) && vi.acts_in_lists(press("Control+c")));
}
