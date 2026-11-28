// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that move cues into groups and out of them, and of
//! copying and cutting a group

use super::{operators::rows, *};
use crate::back::{Cue, group};

/// Returns a show of one list: audio cues 1, 3, 5, and 6, and the groups 2
/// and 4, so that 2 holds 3 and 4, and 4 holds 5
fn grouped(selected: u32) -> App {
    let at = |depth: u8, cue: Cue| Cue { depth, ..cue };
    one_list(
        vec![
            audio(1),
            group(2),
            at(1, audio(3)),
            at(1, group(4)),
            at(2, audio(5)),
            audio(6),
        ],
        Some(CueId(selected)),
    )
}

fn depths(app: &App) -> Vec<u8> {
    app.workspace.lists()[0].depths()
}

#[test]
fn the_keys_of_indent_move_the_selected_cue_into_the_group_above_it() {
    let app = typing(grouped(6), "> >");
    assert_eq!(depths(&app), [0, 0, 1, 1, 2, 1]);
    assert_eq!(app.selected, Some(CueId(6)));
    let outer = typing(grouped(1), "> >");
    assert_eq!(depths(&outer), depths(&grouped(1)));
}

#[test]
fn the_keys_of_outdent_move_the_selected_cue_out_with_the_cues_it_holds() {
    let app = typing(grouped(4), "< <");
    assert_eq!(depths(&app), [0, 0, 1, 0, 1, 0]);
    assert_eq!(rows(&app), [1, 2, 3, 4, 5, 6]);
    assert_eq!(app.selected, Some(CueId(4)));
    assert_eq!(depths(&typing(app, "u")), depths(&grouped(4)));
}

#[test]
fn a_count_or_a_motion_covers_the_cues_from_the_selection_down() {
    let counted = typing(grouped(5), "2 > >");
    assert_eq!(depths(&counted), [0, 0, 1, 1, 2, 1]);
    let reached = typing(grouped(5), "< k");
    assert_eq!(depths(&reached), [0, 0, 1, 0, 1, 0]);
    // The selection stays on the first cue moved
    assert_eq!(reached.selected, Some(CueId(4)));
}

#[test]
fn show_mode_and_the_outputs_leave_the_cues_where_they_are() {
    let shown = locked(grouped(6));
    assert_eq!(depths(&typing(shown, "> >")), depths(&grouped(6)));
    let outputs = after(grouped(6), [Message::ShowOutputs]);
    let kept = typing(typing(outputs, "> >"), "> j");
    assert_eq!(depths(&kept), depths(&grouped(6)));
}

#[test]
fn copying_a_group_takes_the_cues_that_it_holds_once() {
    let app = typing(grouped(2), "y y");
    assert_eq!(held(&app), [2, 3, 4, 5]);
    // A count that reaches a cue that the group holds adds nothing
    let counted = after(grouped(2), [Message::Copy(3)]);
    assert_eq!(held(&counted), [2, 3, 4, 5]);
    let beyond = after(grouped(2), [Message::Copy(5)]);
    assert_eq!(held(&beyond), [2, 3, 4, 5, 6]);
    assert_eq!(on_clipboard(&app), Some(("Copied 4 cues", false)));
}

#[test]
fn copying_a_cue_in_a_group_takes_the_cue_alone() {
    let app = after(grouped(3), [Message::Copy(1)]);
    assert_eq!(held(&app), [3]);
    let nested = after(grouped(4), [Message::Copy(1)]);
    assert_eq!(held(&nested), [4, 5]);
}

#[test]
fn cutting_a_group_copies_the_cues_that_it_holds() {
    let app = after(grouped(2), [Message::Cut(1)]);
    assert_eq!(held(&app), [2, 3, 4, 5]);
    assert_eq!(on_clipboard(&app), Some(("Cut 4 cues", true)));
}
