// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the letters typed in hint mode: those that choose a place, narrow the labels, or take a letter back

use super::*;

#[test]
fn a_label_typed_whole_sends_the_message_of_its_place_and_leaves_the_mode() {
    let chosen = typing(labelled(), "a");
    assert!(chosen.hints.is_none());
    assert!(chosen.outputs_shown);
    let switched = typing(labelled(), "s");
    assert!(switched.hints.is_none());
    assert_eq!(switched.list, 1);
}

#[test]
fn a_label_that_reaches_a_field_starts_a_task_and_leaves_the_mode() {
    let (reached, task) = typing_task(labelled(), "d");
    assert!(reached.hints.is_none());
    assert!(task.units() > 0);
}

#[test]
fn letters_narrow_the_labels_until_one_is_whole_and_backspace_takes_one_back() {
    let begun = typing(many_labelled(), "s");
    assert_eq!(begun.hints.as_ref().map(|hints| hints.typed()), Some("s"));
    assert!(!begun.outputs_shown);
    assert_eq!(
        shown(&begun),
        [
            "s", "a", "d", "f", "j", "k", "l", "e", "w", "c", "m", "p", "g", "h"
        ],
        "the labels that begin with `s` show the letter that follows"
    );
    assert_eq!(shown(&many_labelled()).len(), 20);
    let taken_back = typing(many_labelled(), "s Backspace");
    assert_eq!(
        taken_back.hints.as_ref().map(|hints| hints.typed()),
        Some("")
    );
    // Second letter makes the label whole: the third place, `s d`
    let whole = typing(many_labelled(), "s d");
    assert!(whole.hints.is_none());
    assert_eq!(
        whole.selected,
        Some(CueId(3)),
        "the third place selects row 3"
    );
}

#[test]
fn backspace_with_nothing_typed_leaves_the_mode() {
    assert!(typing(labelled(), "Backspace").hints.is_none());
    assert!(
        typing(many_labelled(), "s Backspace Backspace")
            .hints
            .is_none()
    );
}

#[test]
fn a_character_that_no_label_continues_is_ignored_and_the_mode_stays() {
    let after = typing(labelled(), "k");
    assert!(after.hints.is_some());
    assert_eq!(after.selected, Some(CueId(2)));
    assert_eq!(after.list, 0);
    assert!(!after.outputs_shown);
    let after = typing(many_labelled(), "a h");
    assert_eq!(
        shown(&after),
        shown(&typing(many_labelled(), "a")),
        "`a` stays typed, and `ah` begins none"
    );
    assert_eq!(after.list, 0);
    let after = typing(many_labelled(), "d d 5");
    assert!(after.hints.is_some(), "a slip never runs the next keys");
}

#[test]
fn the_key_of_the_action_is_a_label_letter_in_a_keymap_with_no_cancel_too() {
    let mut own = labelled();
    own.config.keymap = crate::front::keymap::desk("");
    [labelled(), own].into_iter().for_each(|app| {
        let after = typing(app, "f");
        assert!(after.hints.is_some(), "no label of three begins with `f`");
    });
}

#[test]
fn letters_match_the_labels_whatever_their_case() {
    let capital = |text: &str| chord(Key::Character(text.into()), Modifiers::empty());
    let (chosen, _) = labelled().typed(capital("A"), false);
    assert!(chosen.hints.is_none());
    assert!(chosen.outputs_shown);
    let (begun, _) = many_labelled().typed(capital("S"), false);
    assert_eq!(begun.hints.as_ref().map(|hints| hints.typed()), Some("s"));
}
