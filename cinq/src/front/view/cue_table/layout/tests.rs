// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the rows that the cue list shows

use super::Layout;
use crate::back::{Cue, CueId, CueList, Kind, memo};

/// Returns a list of the memo 1, the collapsed group 2 that holds the memos 3
/// and 4 and the group 5, which holds the memo 6, and the memo 7, with standby
/// on cue `standby`
fn list(standby: u32) -> CueList {
    let cue = |id: u32, kind: Kind, depth: u8| {
        Cue::new(CueId(id), id.to_string(), kind, "Cue")
            .depth(depth)
            .collapsed(kind == Kind::Group && id == 2)
            .build()
    };
    CueList {
        standby: Some(CueId(standby)),
        ..CueList::new(
            "Main",
            vec![
                cue(1, Kind::Memo, 0),
                cue(2, Kind::Group, 0),
                cue(3, Kind::Memo, 1),
                cue(4, Kind::Memo, 1),
                cue(5, Kind::Group, 1),
                cue(6, Kind::Memo, 2),
                cue(7, Kind::Memo, 0),
            ],
        )
    }
}

fn ids(cues: Vec<&Cue>) -> Vec<u32> {
    cues.into_iter().map(|cue| cue.id.0).collect()
}

#[test]
fn the_table_draws_the_rows_that_the_list_shows_and_leaves_out_the_cues_a_collapsed_group_holds() {
    let list = list(1);
    assert_eq!(ids(Layout::of(&list).rows()), [1, 2, 7]);
}

#[test]
fn a_hidden_cue_stands_on_the_row_of_the_collapsed_group_that_holds_it() {
    let list = list(1);
    let layout = Layout::of(&list);
    let rows: Vec<_> = [1, 2, 3, 6, 7]
        .map(|id| layout.row_of(CueId(id)))
        .into_iter()
        .collect();
    assert_eq!(rows, [Some(0), Some(1), Some(1), Some(1), Some(2)]);
    assert_eq!(layout.row_of(CueId(99)), None);
}

#[test]
fn the_row_of_standby_reads_next_and_so_does_that_of_the_collapsed_group_that_holds_it() {
    let next = |standby| {
        let list = list(standby);
        let layout = Layout::of(&list);
        list.cues
            .iter()
            .map(|cue| layout.is_next(cue))
            .collect::<Vec<_>>()
    };
    // Standby lies in the collapsed group 2, then on the shown memo 7
    assert_eq!(next(4), [false, true, false, false, false, false, false]);
    assert_eq!(next(7), [false, false, false, false, false, false, true]);
}

#[test]
fn only_a_list_that_holds_a_group_leaves_a_gutter_for_the_mark() {
    let grouped = list(1);
    assert_eq!(Layout::of(&grouped).gutter(), super::INDENT);
    let plain = CueList::new("Main", vec![memo(1)]);
    assert_eq!(Layout::of(&plain).gutter(), 0);
}
