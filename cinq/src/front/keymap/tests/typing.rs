// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of reading keys typed in a row: counts, commands of several keys,
//! the keys of GO and Panic, and keys that act alone

use super::*;
use crate::front::Message;

#[test]
fn counts_repeat_motions() {
    assert_eq!(action("j"), Some((Action::Next, None)));
    assert_eq!(action("1 2 j"), Some((Action::Next, Some(12))));
    assert_eq!(action("3 ArrowUp"), Some((Action::Previous, Some(3))));
    assert_eq!(action("Control+n"), Some((Action::Next, None)));
    assert_eq!(action("2 Control+p"), Some((Action::Previous, Some(2))));
    // Count past the largest that a usize holds
    let most = format!("{}j", "9 ".repeat(20));
    assert_eq!(action(&most), Some((Action::Next, Some(usize::MAX))));
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
    assert_eq!(action("2 d d"), Some((Action::Cut, Some(2))));
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
    let next = |text: &str| -> Vec<(String, Action)> {
        keymap
            .continuations(&keys(text))
            .map(|(rest, action, _)| (written(rest), action))
            .collect()
    };
    let g = [
        ("j".to_owned(), Action::Next),
        ("k".to_owned(), Action::Previous),
        ("g".to_owned(), Action::First),
        ("t".to_owned(), Action::NextTab),
        ("T".to_owned(), Action::PreviousTab),
    ];
    assert_eq!(next("g"), g);
    assert_eq!(next("1 2 g"), g);
    assert_eq!(next("2 d")[0], ("d".to_owned(), Action::Cut));
    assert!(next("3").is_empty());
    assert_eq!(keymap.continuations(&[]).count(), 0);
}

#[test]
fn keys_of_vim_cut_help_first_and_last_take_counts_as_their_twins_do() {
    assert_eq!(action("x"), Some((Action::Cut, None)));
    assert_eq!(action("3 x"), Some((Action::Cut, Some(3))));
    assert_eq!(action("2 Delete"), Some((Action::Cut, Some(2))));
    assert_eq!(action("F1"), Some((Action::Help, None)));
    assert_eq!(action("4 Control+Home"), Some((Action::First, Some(4))));
    assert_eq!(action("Control+End"), Some((Action::Last, None)));
    assert_eq!(action("7 Control+End"), Some((Action::Last, Some(7))));
}

#[test]
fn x_begins_no_other_command_and_types_in_a_field() {
    let keymap = Keymap::default();
    assert!(typed("x").pending.is_empty());
    assert!(keymap.continuations(&keys("x")).next().is_none());
    assert_eq!(
        one("x").and_then(|press| keymap.alone(press)),
        Some(Action::Cut)
    );
}

#[test]
fn keys_of_delete_cut_and_help_stay_with_a_field_that_has_the_keyboard() {
    assert!(!Action::Delete.acts_in_fields());
    assert!(!Action::Cut.acts_in_fields());
    assert!(!Action::Help.acts_in_fields());
}

#[test]
fn counts_of_cua_keys_act_as_those_of_vi_keys_do() {
    let cua = Keymap::from(Preset::Cua);
    let action = |text: &str| {
        keys(text)
            .into_iter()
            .fold(Outcome::default(), |outcome, press| {
                cua.parse(&outcome.pending, press)
            })
            .action
    };
    assert_eq!(action("3 Control+z"), Some((Action::Undo, Some(3))));
    assert_eq!(action("2 Control+y"), Some((Action::Redo, Some(2))));
    assert_eq!(action("2 Control+Tab"), Some((Action::NextTab, Some(2))));
    assert_eq!(
        action("2 Control+Shift+Tab"),
        Some((Action::PreviousTab, Some(2)))
    );
}

#[test]
fn counts_of_vi_keys_reach_undo_redo_and_the_tabs() {
    assert_eq!(action("3 u"), Some((Action::Undo, Some(3))));
    assert_eq!(action("2 Control+r"), Some((Action::Redo, Some(2))));
    assert_eq!(action("2 g t"), Some((Action::NextTab, Some(2))));
    assert_eq!(action("2 g T"), Some((Action::PreviousTab, Some(2))));
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
    let pages = [
        ("Control+d", Action::HalfPageDown),
        ("Control+u", Action::HalfPageUp),
        ("Control+f", Action::PageDown),
        ("PageDown", Action::PageDown),
        ("Control+b", Action::PageUp),
        ("PageUp", Action::PageUp),
    ];
    pages.into_iter().for_each(|(keys, page)| {
        assert_eq!(action(keys), Some((page, None)), "{keys}");
        assert_eq!(
            action(&format!("4 {keys}")),
            Some((page, Some(4))),
            "{keys}"
        );
        assert!(!keymap.acts_in_fields(one(keys).unwrap()), "{keys}");
    });
    // Keys of the prefixes g and d stay theirs, apart from Control+d
    assert_eq!(action("g g"), Some((Action::First, None)));
    assert_eq!(action("d d"), Some((Action::Cut, None)));
    assert_eq!(typed("d").pending.len(), 1);
}

#[test]
fn cua_pages_by_the_whole_page_and_binds_no_key_to_the_half_pages() {
    let keymap = Keymap::from(Preset::Cua);
    assert_eq!(keymap.keys(Action::PageDown), [keys("PageDown")]);
    assert_eq!(keymap.keys(Action::PageUp), [keys("PageUp")]);
    assert!(keymap.keys(Action::HalfPageDown).is_empty());
    assert!(keymap.keys(Action::HalfPageUp).is_empty());
    assert_eq!(keymap.conflicts(), Vec::<String>::new());
}

#[test]
fn vi_copies_with_y_y_cuts_with_d_d_and_pastes_with_p_and_capital_p() {
    assert_eq!(action("y y"), Some((Action::Copy, None)));
    assert_eq!(action("3 y y"), Some((Action::Copy, Some(3))));
    assert_eq!(action("d d"), Some((Action::Cut, None)));
    assert_eq!(action("p"), Some((Action::PasteBelow, None)));
    assert_eq!(action("2 P"), Some((Action::PasteAbove, Some(2))));
    // Cutting is what deleting does in Vim, so that no key only deletes
    assert!(Keymap::default().keys(Action::Delete).is_empty());
}

#[test]
fn cua_copies_cuts_and_pastes_with_control_and_deletes_with_delete_alone() {
    let keymap = Keymap::from(Preset::Cua);
    let action = |text: &str| {
        keymap
            .parse(&[], one(text).expect("a key"))
            .action
            .map(|(action, _)| action)
    };
    assert_eq!(action("Control+c"), Some(Action::Copy));
    assert_eq!(action("Control+x"), Some(Action::Cut));
    assert_eq!(action("Control+v"), Some(Action::PasteBelow));
    assert_eq!(action("Control+V"), Some(Action::PasteAbove));
    assert_eq!(action("Delete"), Some(Action::Delete));
}

#[test]
fn capital_y_copies_like_y_y_and_takes_a_count() {
    assert_eq!(action("Y"), Some((Action::Copy, None)));
    assert_eq!(action("3 Y"), Some((Action::Copy, Some(3))));
    assert_eq!(action("y y"), Some((Action::Copy, None)));
    assert!(typed("Y").pending.is_empty());
    // Y is bound by itself, so it makes no operator
    assert_eq!(typed("Y j").action, Some((Action::Next, None)));
}

#[test]
fn counts_reach_the_actions_that_add_cues() {
    assert_eq!(action("3 o"), Some((Action::AddBelow, Some(3))));
    assert_eq!(action("2 O"), Some((Action::AddAbove, Some(2))));
    assert!(matches!(
        Action::AddBelow.message(Some(3)),
        Message::Add(_, 3)
    ));
    assert!(matches!(Action::AddAbove.message(None), Message::Add(_, 1)));
}
