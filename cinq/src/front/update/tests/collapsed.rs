// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys and counts that move through the rows of a list with a
//! collapsed group, which count the rows that the table shows, and of the
//! selection, which a collapsed group never hides

use super::{operators::rows, *};
use crate::{
    back::{Cue, Kind},
    front::message::Fold,
};

/// Returns a show of one list with the group 2 collapsed: audio cues 1, 7,
/// and 8, and the group 2, which holds 3, the group 4 with 5 in it, and 6
pub(super) fn collapsed(selected: u32) -> App {
    let cue = |id: u32, depth: u8, kind: Kind, name: &str| Cue {
        depth,
        ..Cue::new(CueId(id), id.to_string(), kind, name).build()
    };
    let cues = vec![
        cue(1, 0, Kind::Audio, "Rain"),
        Cue {
            collapsed: true,
            ..cue(2, 0, Kind::Group, "Storm")
        },
        cue(3, 1, Kind::Audio, "Thunder"),
        cue(4, 1, Kind::Group, "Wind"),
        cue(5, 2, Kind::Audio, "Gust"),
        cue(6, 1, Kind::Audio, "Lightning"),
        cue(7, 0, Kind::Audio, "Birds"),
        cue(8, 0, Kind::Audio, "Dawn"),
    ];
    one_list(cues, Some(CueId(selected)))
}

/// Returns the state after the groups open or close, as a menu does
pub(super) fn folded(app: App, fold: Fold) -> App {
    after(app, [Message::Fold(fold)])
}

pub(super) fn selected(app: &App) -> Option<u32> {
    app.selected.map(|id| id.0)
}

pub(super) fn closed(app: &App, id: u32) -> bool {
    app.workspace
        .cue(CueId(id))
        .is_some_and(|cue| cue.collapsed)
}

#[test]
fn the_table_shows_one_row_for_a_collapsed_group() {
    let app = collapsed(1);
    assert_eq!(app.row_count(), 4);
    assert_eq!(app.row(), Some(0));
    assert_eq!(app.selecting(Some(2)).selected, Some(CueId(7)));
    assert_eq!(collapsed(2).row(), Some(1));
    assert_eq!(collapsed(7).row(), Some(2));
}

#[test]
fn a_cue_that_the_group_hides_is_on_the_row_of_the_group() {
    assert_eq!(collapsed(5).row(), Some(1));
}

#[test]
fn the_keys_that_step_and_jump_count_rows_that_are_shown() {
    assert_eq!(selected(&typing(collapsed(1), "j")), Some(2));
    assert_eq!(selected(&typing(collapsed(1), "2 j")), Some(7));
    assert_eq!(selected(&typing(collapsed(8), "2 k")), Some(2));
    assert_eq!(selected(&typing(collapsed(1), "G")), Some(8));
    assert_eq!(selected(&typing(collapsed(1), "3 G")), Some(7));
    assert_eq!(selected(&typing(collapsed(1), "2 G")), Some(2));
    assert_eq!(selected(&typing(collapsed(8), "g g")), Some(1));
    assert_eq!(selected(&typing(collapsed(1), "9 G")), Some(8));
}

#[test]
fn deleting_a_collapsed_group_takes_every_cue_that_it_holds() {
    let app = typing(collapsed(2), "d d");
    assert_eq!(rows(&app), [1, 7, 8]);
    assert_eq!(selected(&app), Some(7));
}

#[test]
fn a_count_or_a_motion_takes_the_rows_that_are_shown() {
    assert_eq!(rows(&typing(collapsed(1), "2 d d")), [7, 8]);
    assert_eq!(rows(&typing(collapsed(1), "d 2 j")), [8]);
    assert_eq!(rows(&typing(collapsed(2), "d G")), [1]);
    assert_eq!(rows(&typing(collapsed(8), "d 2 k")), [1]);
    assert_eq!(rows(&typing(collapsed(1), "d 3 G")), [8]);
}

#[test]
fn copying_and_cutting_a_collapsed_group_take_every_cue_that_it_holds() {
    let copied = typing(collapsed(1), "2 y y");
    assert_eq!(held(&copied), [1, 2, 3, 4, 5, 6]);
    let cut = typing(collapsed(2), "x");
    assert_eq!(rows(&cut), [1, 7, 8]);
    assert_eq!(held(&cut), [2, 3, 4, 5, 6]);
}

#[test]
fn indenting_the_rows_counts_the_group_as_one() {
    // The cue below the group goes in as its last cue, and the group opens
    let app = typing(collapsed(7), "> >");
    assert_eq!(app.workspace.lists()[0].depths(), [0, 0, 1, 1, 2, 1, 1, 0]);
    assert_eq!(selected(&app), Some(7));
    assert!(!closed(&app, 2));
    // A cue that no group lies above leaves the group closed
    assert!(closed(&typing(collapsed(1), "> >"), 2));
    // A count takes the rows below as well
    let nested = typing(collapsed(7), "2 > >");
    assert_eq!(
        nested.workspace.lists()[0].depths(),
        [0, 0, 1, 1, 2, 1, 1, 1]
    );
}
