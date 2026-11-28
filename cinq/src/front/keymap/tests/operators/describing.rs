// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the descriptions of the motions and rows after an operator

use super::*;

#[test]
fn box_lists_the_motions_after_an_operator_with_what_they_do() {
    let keymap = Keymap::default();
    let next = |text: &str| following(&keymap, text);
    let after_d = next("d");
    assert_eq!(after_d[0], ("d".to_owned(), Action::Cut, None));
    let motions: Vec<_> = after_d
        .iter()
        .filter(|(_, _, operator)| operator.is_some())
        .map(|(rest, action, operator)| (rest.as_str(), *action, *operator))
        .collect();
    let cut = Some(Action::Cut);
    assert_eq!(
        motions,
        [
            ("j", Action::Next, cut),
            ("g j", Action::Next, cut),
            ("ArrowDown", Action::Next, cut),
            ("Control+n", Action::Next, cut),
            ("+", Action::Next, cut),
            ("k", Action::Previous, cut),
            ("g k", Action::Previous, cut),
            ("ArrowUp", Action::Previous, cut),
            ("Control+p", Action::Previous, cut),
            ("-", Action::Previous, cut),
            ("g g", Action::First, cut),
            ("Control+Home", Action::First, cut),
            ("G", Action::Last, cut),
            ("Control+End", Action::Last, cut),
        ]
    );
    // Count in the middle changes nothing, and half a motion narrows it
    assert_eq!(next("2 d 3"), after_d);
    assert_eq!(
        next("d g"),
        [
            ("j".to_owned(), Action::Next, cut),
            ("k".to_owned(), Action::Previous, cut),
            ("g".to_owned(), Action::First, cut),
        ]
    );
    assert_eq!(
        next("y g")[2],
        ("g".to_owned(), Action::First, Some(Action::Copy))
    );
}

#[test]
fn descriptions_name_the_operator_and_the_rows_it_reaches() {
    assert_eq!(
        Action::Cut.over(Action::Next, Rows::Cues, None),
        "Cut the selected cue and the next"
    );
    assert_eq!(
        Action::Copy.over(Action::Last, Rows::Cues, None),
        "Copy the cues from the selection down to the last"
    );
    assert_eq!(
        Action::Delete.over(Action::First, Rows::Outputs, None),
        "Delete the outputs from the selection up to the first"
    );
    assert_eq!(
        Action::Cut.over(Action::Previous, Rows::Outputs, None),
        "Cut the selected output and the previous"
    );
}

#[test]
fn descriptions_name_the_rows_that_show_and_never_both() {
    let (cues, outputs) = (Rows::Cues, Rows::Outputs);
    assert_eq!(
        Action::AddBelow.description_in(cues),
        "Add a cue below the selection"
    );
    assert_eq!(
        Action::AddBelow.description_in(outputs),
        "Add an output below the selection"
    );
    assert_eq!(
        Action::PasteAbove.description_in(outputs),
        "Paste the copied outputs above the selection"
    );
    assert_eq!(
        Action::ChangeName.description_in(outputs),
        "Change the name of the selected output, selecting all of it"
    );
    assert_eq!(
        Action::Next.description_in(cues),
        "Select the next cue or menu option"
    );
    assert_eq!(
        Action::Search.description_in(outputs),
        "Search the outputs by the text of their columns"
    );
    Action::VARIANTS.iter().for_each(|action| {
        [cues, outputs].into_iter().for_each(|rows| {
            let text = action.description_in(rows);
            assert!(
                !text.contains("or output") && !text.contains("or the output"),
                "{text}"
            );
        });
    });
}

#[test]
fn help_page_names_the_keys_that_begin_an_operator() {
    assert_eq!(Keymap::default().operator_keys(), ["d", "y", ">", "<"]);
    assert!(declared("next = \"ArrowDown\"").operator_keys().is_empty());
}

#[test]
fn descriptions_follow_the_count_typed_so_far() {
    let keymap = Keymap::default();
    let pending = |text| keymap.pending_count(&keys(text));
    assert_eq!(pending("d"), None);
    assert_eq!(pending("3 d"), Some(3));
    assert_eq!(pending("2 d 3"), Some(6));
    assert_eq!(pending("d 4"), Some(4));
    assert_eq!(pending("3 g"), Some(3));
    assert_eq!(
        Action::Cut.over(Action::Next, Rows::Cues, Some(6)),
        "Cut the selected cue and the next 6"
    );
    assert_eq!(
        Action::Copy.over(Action::Last, Rows::Cues, Some(3)),
        "Copy the cues from the selection to row 3"
    );
}
