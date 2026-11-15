// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the key bindings, with the helpers that read the keys they
//! type

mod file;
mod manual;
mod notation;
mod typing;

use itertools::Itertools;
use strum::VariantArray;

use super::{
    Action, Keymap, Preset, Press, binding, config,
    press::{sequence, written},
    typing::Outcome,
};

fn keys(text: &str) -> Vec<Press> {
    sequence(text).expect("keys in the tests are valid")
}

fn one(text: &str) -> Option<Press> {
    keys(text).first().copied()
}

/// Returns what the last key amounts to
fn typed(text: &str) -> Outcome {
    let keymap = Keymap::default();
    keys(text)
        .into_iter()
        .fold(Outcome::default(), |outcome, press| {
            keymap.parse(&outcome.pending, press)
        })
}

fn action(text: &str) -> Option<(Action, Option<usize>)> {
    typed(text).action
}

#[test]
fn shipped_keymaps_are_valid_and_hide_no_binding() {
    let invalid: Vec<_> = Preset::VARIANTS
        .iter()
        .flat_map(|preset| Action::VARIANTS.iter().map(|&action| preset.keys(action)))
        .flatten()
        .copied()
        .filter(|keys| binding(keys).is_err())
        .collect();
    assert!(invalid.is_empty(), "{invalid:?}");
    let conflicts: Vec<_> = Preset::VARIANTS
        .iter()
        .map(|&preset| (preset.name(), Keymap::from(preset).conflicts()))
        .filter(|(_, conflicts)| !conflicts.is_empty())
        .collect();
    assert!(conflicts.is_empty(), "{conflicts:?}");
}

#[test]
fn open_is_control_l_in_vi_and_control_o_in_cua() {
    let opens = |preset: Preset| {
        Keymap::from(preset)
            .key(Action::Open)
            .expect("both keymaps bind open")
    };
    assert_eq!(opens(Preset::Vi), "Control+l");
    assert_eq!(opens(Preset::Cua), "Control+o");
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
            "`Space` acts at once as go, so `Shift+Space` can never be typed",
            "`Space` acts at once as go, so `g Space` can never be typed",
        ]
    );
}

#[test]
fn back_and_forward_keys_follow_vim_and_the_browsers_and_act_in_fields() {
    let keys = |preset: Preset, action: Action| Keymap::from(preset).listed(action);
    assert_eq!(keys(Preset::Vi, Action::Back), "Control+o");
    assert_eq!(keys(Preset::Vi, Action::Forward), "Control+i");
    assert_eq!(keys(Preset::Cua, Action::Back), "Alt+ArrowLeft");
    assert_eq!(keys(Preset::Cua, Action::Forward), "Alt+ArrowRight");
    let acting = |preset: Preset, text: &str| {
        one(text).is_some_and(|press| Keymap::from(preset).acts_in_fields(press))
    };
    assert!(acting(Preset::Vi, "Control+o") && acting(Preset::Vi, "Control+i"));
    assert!(acting(Preset::Cua, "Alt+ArrowLeft") && acting(Preset::Cua, "Alt+ArrowRight"));
    assert!(!acting(Preset::Cua, "Control+i"));
}

#[test]
fn no_key_is_bound_twice_in_a_shipped_keymap() {
    let twice: Vec<_> = Preset::VARIANTS
        .iter()
        .flat_map(|preset| {
            Action::VARIANTS
                .iter()
                .flat_map(|&action| preset.keys(action))
                .duplicates()
                .map(|keys| (preset.name(), *keys))
        })
        .collect();
    assert!(twice.is_empty(), "{twice:?}");
}
