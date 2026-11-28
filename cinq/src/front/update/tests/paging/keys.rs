// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys of a page in the cue list and on the help page: what they
//! start, what a landing does, and the rows of a half page that a count sets

use iced::{
    Size,
    keyboard::{Key, key::Named},
    window,
};

use super::{
    super::{helping, press, show},
    letter, typed,
};
use crate::{
    back::CueId,
    front::{App, Message, Spot},
};

#[test]
fn keys_of_a_page_measure_the_cue_list_before_the_selection_moves() {
    let app = show(Some(CueId(2)));
    let (after, started) = typed(app, "d", &[]);
    assert_eq!((after.selected, started), (Some(CueId(2)), 2));
    let (after, started) = typed(after, "f", &[]);
    assert_eq!((after.selected, started), (Some(CueId(2)), 2));
}

#[test]
fn count_before_half_a_page_sets_how_many_rows_it_moves_from_then_on() {
    let (counted, _) = typed(show(Some(CueId(2))), "d", &["5"]);
    assert_eq!(counted.scroll, Some(5));
    let (kept, _) = typed(counted, "u", &[]);
    assert_eq!(kept.scroll, Some(5));
    // Count before a whole page counts pages, so it sets nothing
    let (pages, _) = typed(show(Some(CueId(2))), "f", &["5"]);
    assert_eq!(pages.scroll, None);
}

#[test]
fn landing_selects_the_row_without_a_jump_or_going_past_the_end() {
    let landed = |row| landing_at(show(Some(CueId(2))), 0, Some(CueId(2)), row);
    assert_eq!(landed(7).selected, Some(CueId(8)));
    assert_eq!(landed(0).selected, Some(CueId(1)));
    assert_eq!(landed(usize::MAX).selected, Some(CueId(12)));
    assert!(!landed(7).can_go_back());
}

fn landing_at(app: App, list: usize, from: Option<CueId>, row: usize) -> App {
    let from = from.map_or(Spot::List(list), Spot::Cue);
    app.next(Message::Landed { from, row }).0
}

#[test]
fn landing_is_dropped_when_a_key_changed_the_list_or_the_selection_meanwhile() {
    let moved = landing_at(show(Some(CueId(5))), 0, Some(CueId(2)), 7);
    assert_eq!(moved.selected, Some(CueId(5)));
    let other = App {
        list: 1,
        ..show(Some(CueId(20)))
    };
    assert_eq!(landing_at(other, 0, Some(CueId(20)), 0).list, 1);
}

#[test]
fn landing_keeps_a_command_typed_halfway() {
    let (halfway, _) = show(Some(CueId(2))).typed(press(letter("5")), false);
    let landed = landing_at(halfway, 0, Some(CueId(2)), 7);
    assert_eq!(landed.keys.len(), 1);
    assert_eq!(landed.selected, Some(CueId(8)));
}

#[test]
fn rows_of_a_half_page_that_a_count_set_reset_as_the_main_window_changes_size() {
    let main = window::Id::unique();
    let counted = || App {
        main: Some(main),
        scroll: Some(5),
        ..show(Some(CueId(2)))
    };
    let size = Size::new(800.0, 600.0);
    let (other, _) = counted().next(Message::Resized(window::Id::unique(), size));
    assert_eq!(other.scroll, Some(5));
    let (resized, _) = counted().next(Message::Resized(main, size));
    assert_eq!(resized.scroll, None);
}

#[test]
fn keys_of_a_page_scroll_the_help_page_and_move_no_selection() {
    let open = || helping(show(Some(CueId(2))));
    ["d", "u", "f", "b"].into_iter().for_each(|key| {
        let (after, started) = typed(open(), key, &[]);
        assert_eq!(
            (after.selected, after.scroll, started),
            (Some(CueId(2)), None, 2)
        );
    });
    // Count sets the lines of a half page, which the cue list shares
    let (after, started) = typed(open(), "d", &["4"]);
    assert_eq!((after.scroll, started), (Some(4), 2));
    let (after, _) = typed(after, "f", &["2"]);
    assert_eq!(after.scroll, Some(4));
}

#[test]
fn named_keys_of_a_page_act_in_the_cue_list_and_on_the_help_page() {
    let page = |help: bool, named: Named| {
        let app = App {
            help,
            ..show(Some(CueId(2)))
        };
        let (_, task) = app.typed(press(Key::Named(named)), false);
        task.units()
    };
    for help in [false, true] {
        assert_eq!(
            [Named::PageDown, Named::PageUp].map(|n| page(help, n)),
            [2, 2],
            "help {help}"
        );
    }
}
