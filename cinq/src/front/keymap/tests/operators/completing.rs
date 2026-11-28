// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how an operator completes a command, with counts and interruptions

use super::*;

#[test]
fn operator_and_motion_complete_a_command_through_every_key_of_the_motion() {
    let motions = [
        (Action::Next, ["j", "ArrowDown", "Control+n"].as_slice()),
        (Action::Previous, &["k", "ArrowUp", "Control+p"]),
        (Action::First, &["g g", "Control+Home"]),
        (Action::Last, &["G", "Control+End"]),
    ];
    for (operator, key) in [(Action::Cut, "d"), (Action::Copy, "y")] {
        for (motion, bound) in motions {
            for keys in bound {
                let text = format!("{key} {keys}");
                assert_eq!(across(&text), Some((operator, motion, None)), "{text}");
            }
        }
    }
}

#[test]
fn counts_before_the_operator_or_the_motion_or_both_multiply() {
    let most = format!("d {}j", "9 ".repeat(20));
    assert_across(&[
        ("3 d j", Action::Cut, Action::Next, Some(3)),
        ("d 3 j", Action::Cut, Action::Next, Some(3)),
        ("2 d 3 j", Action::Cut, Action::Next, Some(6)),
        ("1 2 y 2 k", Action::Copy, Action::Previous, Some(24)),
        ("d 3 G", Action::Cut, Action::Last, Some(3)),
        ("d 4 g g", Action::Cut, Action::First, Some(4)),
        (&most, Action::Cut, Action::Next, Some(usize::MAX)),
    ]);
    // Parts stay apart in the outcome
    let outcome = typed("2 d 3 j");
    assert_eq!(outcome.action, Some((Action::Cut, Some(2))));
    assert_eq!(outcome.motion, Some((Action::Next, Some(3))));
}

#[test]
fn doubled_operator_multiplies_the_counts_too() {
    assert_actions(&[
        ("d d", Action::Cut, None),
        ("3 d d", Action::Cut, Some(3)),
        ("d 3 d", Action::Cut, Some(3)),
        ("2 d 3 d", Action::Cut, Some(6)),
        ("2 y 3 y", Action::Copy, Some(6)),
    ]);
    assert_eq!(typed("2 d 3 d").motion, None);
}

#[test]
fn operator_waits_for_a_count_and_for_the_rest_of_a_motion() {
    ["d", "d 3", "2 d 3", "d g", "2 d 3 g"]
        .into_iter()
        .for_each(|text| assert_eq!(typed(text).pending, keys(text), "{text}"));
    assert!(typed("d g").action.is_none() && typed("d 3").action.is_none());
    assert!(typed("d j").pending.is_empty());
}

#[test]
fn go_and_panic_act_at_once_in_the_middle_of_an_operator() {
    let outcome = typed("d 3 Space");
    assert_eq!(outcome.action, Some((Action::Go, None)));
    assert!(outcome.pending.is_empty() && outcome.motion.is_none());
    assert_eq!(action("2 y Escape"), Some((Action::Panic, None)));
}
