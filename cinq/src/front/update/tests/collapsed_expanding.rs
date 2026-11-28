// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys and places that expand a collapsed group, which a
//! selection, standby, or a jump that reaches a hidden cue opens

use super::{
    collapsed::{closed, collapsed, folded, selected},
    operators::rows,
    *,
};
use crate::{
    back::{Command, Cue, Kind},
    front::message::Fold,
};

#[test]
fn collapsing_and_expanding_keep_the_selection_on_a_shown_row() {
    let hidden = folded(collapsed(5), Fold::OpenAll);
    assert_eq!(selected(&hidden), Some(5));
    let again = folded(hidden, Fold::CloseAll);
    assert_eq!(selected(&again), Some(2));
    assert_eq!(again.row(), Some(1));
}

#[test]
fn selecting_a_cue_that_a_group_hides_expands_the_groups_that_hide_it() {
    let nested = folded(collapsed(1), Fold::CloseAll);
    assert!(closed(&nested, 2) && closed(&nested, 4));
    let app = after(nested, [Message::Select(CueId(5))]);
    assert_eq!(selected(&app), Some(5));
    assert!(!closed(&app, 2) && !closed(&app, 4));
    assert_eq!(app.row(), Some(4));
}

#[test]
fn undo_that_selects_a_hidden_cue_expands_its_group() {
    let deleted = typing(collapsed(1), "j z o j d d");
    assert_eq!(rows(&deleted), [1, 2, 4, 5, 6, 7, 8]);
    let folded = folded(deleted, Fold::CloseAll);
    assert!(closed(&folded, 2));
    let undone = after(folded, [Message::Undo(1)]);
    assert_eq!(selected(&undone), Some(3));
    assert!(!closed(&undone, 2));
}

#[test]
fn the_jump_list_expands_the_group_that_hides_where_it_goes() {
    let open = folded(collapsed(1), Fold::OpenAll);
    let jumped = typing(typing(open, "5 G"), "g g");
    assert_eq!(selected(&jumped), Some(1));
    let folded = folded(jumped, Fold::CloseAll);
    let back = typing(folded, "Control+o");
    assert_eq!(selected(&back), Some(5));
    assert!(!closed(&back, 2) && !closed(&back, 4));
}

#[test]
fn go_that_moves_the_selection_with_standby_into_a_group_expands_it() {
    let target = |app: App| {
        let goto = Cue::new(CueId(9), "9", Kind::GoTo, "Back")
            .target(CueId(5))
            .build();
        let list = CueList::new(
            "Main",
            app.workspace.lists()[0]
                .cues
                .iter()
                .cloned()
                .chain([goto])
                .collect(),
        );
        App {
            workspace: Workspace::new(vec![list]),
            selected: Some(CueId(9)),
            ..app
        }
        .send(Command::Standby {
            list: 0,
            cue: Some(CueId(9)),
        })
    };
    let app = after(target(collapsed(9)), [Message::Go]);
    assert_eq!(selected(&app), Some(5));
    assert!(!closed(&app, 2));
}

#[test]
fn standby_may_stay_in_a_collapsed_group_with_no_selection_in_it() {
    let app = collapsed(1).send(Command::Standby {
        list: 0,
        cue: Some(CueId(5)),
    });
    assert_eq!(app.current().standby, Some(CueId(5)));
    assert!(closed(&app, 2));
}

#[test]
fn a_page_or_a_scroll_lands_on_a_row_that_is_shown() {
    let from = collapsed(1).spot();
    let landed = after(collapsed(1), [Message::Landed { from, row: 1 }]);
    assert_eq!(selected(&landed), Some(2));
    let reached = after(collapsed(1), [Message::Reached { from, row: 3 }]);
    assert_eq!(selected(&reached), Some(8));
}

#[test]
fn a_row_number_counts_the_rows_that_are_shown() {
    assert_eq!(selected(&after(collapsed(1), [Message::Row(3)])), Some(7));
    assert_eq!(selected(&after(collapsed(1), [Message::Row(99)])), Some(8));
}
