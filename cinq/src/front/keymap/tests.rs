// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the key bindings, with the helpers that read the keys they type

mod cancel;
mod dropping;
mod file;
mod manual;
mod naming;
mod notation;
mod operators;
mod typing;
mod viewing;

use itertools::Itertools;
use strum::VariantArray;

use super::{
    Action, Keymap, Press, Rows, binding, declared, desk,
    press::{sequence, shown, written},
    typing::Outcome,
    vi,
};
use crate::front::config;

fn keys(text: &str) -> Vec<Press> {
    sequence(text).expect("keys in the tests are valid")
}

fn one(text: &str) -> Option<Press> {
    keys(text).first().copied()
}

fn press(text: &str) -> Press {
    one(text).expect("a key")
}

fn listed(action: Action) -> String {
    Keymap::default().listed(action)
}

/// Returns what the last key amounts to on a keymap
fn typed_on(keymap: &Keymap, text: &str) -> Outcome {
    keys(text)
        .into_iter()
        .fold(Outcome::default(), |outcome, press| {
            keymap.parse(&outcome.pending, press)
        })
}

fn typed(text: &str) -> Outcome {
    typed_on(&Keymap::default(), text)
}

fn action(text: &str) -> Option<(Action, Option<usize>)> {
    typed(text).action
}

/// Returns what can follow the keys typed so far, as the box of the help
/// page lists it, with the operator that a motion completes
fn following(keymap: &Keymap, text: &str) -> Vec<(String, Action, Option<Action>)> {
    keymap
        .continuations(&keys(text))
        .map(|(rest, action, operator)| (written(rest), action, operator))
        .collect()
}

fn assert_actions(wanted: &[(&str, Action, Option<usize>)]) {
    wanted.iter().for_each(|&(text, action, count)| {
        assert_eq!(typed(text).action, Some((action, count)), "{text}");
    });
}

#[test]
fn the_shipped_keymap_is_valid_and_hides_no_binding() {
    let invalid: Vec<_> = Action::VARIANTS
        .iter()
        .flat_map(|&action| vi::keys(action))
        .copied()
        .filter(|keys| binding(keys).is_err())
        .collect();
    assert!(invalid.is_empty(), "{invalid:?}");
    let conflicts = Keymap::default().conflicts();
    assert!(conflicts.is_empty(), "{conflicts:?}");
}

#[test]
fn open_is_control_l_in_vi_since_control_o_goes_back() {
    assert_eq!(listed(Action::Open), "Control+l");
    assert_eq!(listed(Action::Back), "Control+o");
}

#[test]
fn keys_of_go_and_panic_hide_every_other_sequence_that_holds_them() {
    let keymap = Keymap::default()
        .with(Action::Panic, vec![keys("Escape"), keys("Shift+Escape")])
        .with(Action::NextList, vec![keys("Shift+Space")])
        .with(Action::First, vec![keys("g Space")])
        .with(Action::Delete, vec![keys("Space x")]);
    assert_eq!(
        keymap.conflicts(),
        [
            "`Space` comes first, so `Space x` can never be typed",
            "`Space` acts at once as go, so `g Space` can never be typed",
            "`Space` acts at once as go, so `Shift+Space` can never be typed",
        ]
    );
}

#[test]
fn back_and_forward_keys_follow_vim_and_the_browsers_and_act_in_fields() {
    assert_eq!(listed(Action::Forward), "Control+i");
    let acting = |text: &str| Keymap::default().acts_in_fields(press(text));
    assert!(acting("Control+o") && acting("Control+i"));
}

#[test]
fn no_key_is_bound_twice_in_one_place_of_the_shipped_keymap() {
    let places: [fn(Action) -> bool; 2] = [Action::acts_in_cue_list, Action::acts_in_menu];
    let twice: Vec<_> = places
        .into_iter()
        .flat_map(|acts_in| {
            Action::VARIANTS
                .iter()
                .filter(move |&&action| acts_in(action))
                .flat_map(|&action| vi::keys(action))
                .duplicates()
        })
        .collect();
    assert!(twice.is_empty(), "{twice:?}");
}

#[test]
fn question_mark_and_f1_show_the_help_and_search_up_has_no_key() {
    assert_eq!(listed(Action::Help), "?, F1");
    assert_eq!(listed(Action::SearchBackward), "none");
    let keymap = Keymap::default();
    assert_eq!(keymap.alone(press("?")), Some(Action::Help));
    assert!(!keymap.acts_in_fields(press("?")));
    assert!(!keymap.acts_when_captured(press("?"), true));
}

#[test]
fn a_key_may_pick_in_a_menu_and_scroll_the_cue_list() {
    let keymap = Keymap::default();
    let menu = |text: &str| keymap.in_menu(press(text));
    assert_actions(&[
        ("Control+e", Action::ScrollDown, None),
        ("3 Control+y", Action::ScrollUp, Some(3)),
    ]);
    assert_eq!(menu("Control+e"), Some(Action::CloseMenu));
    assert_eq!(menu("Control+y"), Some(Action::Pick));
    // Keys of the list are no keys of a menu
    assert_eq!(menu("Control+d"), None);
}

#[test]
fn a_clash_in_the_cue_list_or_in_a_menu_is_still_reported() {
    let in_list = Keymap::default().with(Action::Next, vec![keys("j"), keys("Control+e")]);
    assert_eq!(
        in_list.conflicts(),
        [
            "`Control+e` is bound to both next and scroll_down, so next keeps it and scroll_down loses it",
            "`Control+e` is bound to both next and close_menu, so next keeps it and close_menu loses it",
        ]
    );
    let in_menu = Keymap::default().with(Action::Pick, vec![keys("Control+e")]);
    assert_eq!(
        in_menu.conflicts(),
        [
            "`Control+e` is bound to both pick and close_menu, so pick keeps it and close_menu loses it"
        ]
    );
    let across = Keymap::default().with(Action::Pick, vec![keys("j")]);
    assert_eq!(
        across.conflicts(),
        ["`j` is bound to both next and pick, so next keeps it and pick loses it"]
    );
}

#[test]
fn a_clash_over_g_j_g_k_and_the_search_keys_is_found_in_the_place_they_act_in() {
    let with = |action, text: &str| Keymap::default().with(action, vec![keys(text)]);
    assert_eq!(
        with(Action::ScrollDown, "g j").conflicts(),
        ["`g j` is bound to both next and scroll_down, so next keeps it and scroll_down loses it"]
    );
    assert_eq!(
        with(Action::ScrollUp, "/").conflicts(),
        ["`/` is bound to both scroll_up and search, so scroll_up keeps it and search loses it"]
    );
    assert_eq!(
        with(Action::Pick, "g k").conflicts(),
        ["`g k` is bound to both previous and pick, so previous keeps it and pick loses it"]
    );
    // Search keys act in the cue list only, so a menu may use them
    assert_eq!(with(Action::Pick, "/").conflicts(), Vec::<String>::new());
    assert_eq!(
        with(Action::CloseMenu, "n").conflicts(),
        Vec::<String>::new()
    );
}
