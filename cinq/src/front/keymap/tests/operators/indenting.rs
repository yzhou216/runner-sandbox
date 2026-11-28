// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of indent and outdent, which are operators

use super::*;

#[test]
fn indent_and_outdent_take_counts_and_motions_as_cut_and_copy_do() {
    assert_actions(&[
        ("> >", Action::Indent, None),
        ("3 > >", Action::Indent, Some(3)),
        ("> 3 >", Action::Indent, Some(3)),
        ("< <", Action::Outdent, None),
        ("2 < 3 <", Action::Outdent, Some(6)),
    ]);
    assert_across(&[
        ("> j", Action::Indent, Action::Next, None),
        ("3 > k", Action::Indent, Action::Previous, Some(3)),
        ("< G", Action::Outdent, Action::Last, None),
        ("< 4 g g", Action::Outdent, Action::First, Some(4)),
    ]);
    assert!(typed("> x").pending.is_empty() && typed("> x").action.is_none());
}

#[test]
fn descriptions_name_the_move_of_indent_and_outdent() {
    assert_eq!(
        Action::Indent.description(),
        "Move the selected cue into the group above it"
    );
    assert_eq!(
        Action::Outdent.description(),
        "Move the selected cue out of its group"
    );
    assert_eq!(
        Action::Indent.over(Action::Next, Rows::Cues, None),
        "Indent the selected cue and the next"
    );
    assert_eq!(
        Action::Outdent.over(Action::Last, Rows::Cues, None),
        "Outdent the cues from the selection down to the last"
    );
}

#[test]
fn indent_and_outdent_bound_to_single_keys_take_counts_and_make_no_operator() {
    let keymap = declared("indent = \"Control+]\"\noutdent = \"Control+[\"");
    assert_eq!(
        typed_on(&keymap, "Control+]").action,
        Some((Action::Indent, None))
    );
    assert_eq!(
        typed_on(&keymap, "3 Control+[").action,
        Some((Action::Outdent, Some(3)))
    );
    assert!(keymap.operator_keys().is_empty());
}
