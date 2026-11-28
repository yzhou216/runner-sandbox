// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of adding cues, one or as many as a count types

use super::{operators::rows, *};
use crate::front::app::{Held, Keyboard};

#[test]
fn one_cue_added_below_is_selected_as_before() {
    let app = typing(show(Some(CueId(4))), "o");
    assert_eq!(rows(&app)[3..6], [4, 22, 5]);
    assert_eq!(app.selected, Some(CueId(22)));
}

#[test]
fn count_adds_that_many_cues_below_in_one_change_and_selects_the_last() {
    let app = typing(show(Some(CueId(4))), "3 o");
    assert_eq!(rows(&app)[3..8], [4, 22, 23, 24, 5]);
    // Vim leaves the cursor on the last line that `3o` inserts
    assert_eq!(app.selected, Some(CueId(24)));
    let undone = typing(app, "u");
    assert_eq!(rows(&undone), (1..=12).collect::<Vec<_>>());
    // Back on the cue that was selected, as `o u` is in Vim
    assert_eq!(undone.selected, Some(CueId(4)));
}

#[test]
fn count_adds_that_many_cues_above_and_selects_the_last_and_one_undo_takes_all_of_them() {
    let app = typing(show(Some(CueId(4))), "2 O");
    assert_eq!(rows(&app)[2..7], [3, 22, 23, 4, 5]);
    assert_eq!(app.selected, Some(CueId(23)));
    // One undo takes all of a count
    let app = typing(show(Some(CueId(4))), "3 O");
    assert_eq!(rows(&app)[2..7], [3, 22, 23, 24, 4]);
    let undone = typing(app, "u");
    assert_eq!(rows(&undone), (1..=12).collect::<Vec<_>>());
    assert_eq!(undone.selected, Some(CueId(4)));
}

#[test]
fn count_without_a_selection_adds_at_the_end() {
    let app = typing(show(None), "2 o");
    assert_eq!(rows(&app)[10..], [11, 12, 22, 23]);
    assert_eq!(app.selected, Some(CueId(23)));
}

#[test]
fn add_stops_at_a_thousand_cues() {
    let (app, _) = show(Some(CueId(4))).handled(Message::Add(Place::After, usize::MAX));
    assert_eq!(rows(&app).len(), 12 + 1000);
}

#[test]
fn y_copies_the_selected_cue_and_takes_a_count() {
    let app = typing(show(Some(CueId(4))), "Y p");
    assert_eq!(rows(&app)[3..6], [4, 22, 5]);
    let app = typing(show(Some(CueId(4))), "2 Y p");
    assert_eq!(rows(&app)[3..7], [4, 22, 23, 5]);
}

#[test]
fn show_mode_tells_why_a_key_that_edits_did_nothing_and_stays_quiet_for_the_rest() {
    let told = |keys| typing(locked(show(Some(CueId(4)))), keys).notices.remark;
    assert!(
        ["o", "p", "x", "d d", "c c", "d j"]
            .into_iter()
            .all(|keys| told(keys) == Some("Leave show mode to edit"))
    );
    // Copying and moving about still work
    assert!(
        ["Y", "y y", "j", "k"]
            .into_iter()
            .all(|keys| told(keys).is_none())
    );
}

#[test]
fn the_show_mode_check_box_flips_both_ways_since_show_mode_never_locks_it() {
    let closed = locked(show(Some(CueId(4))));
    assert!(closed.workspace.show_mode() && !closed.editable());
    let (unlocked, _) = closed.handled(Message::ShowMode(false));
    assert!(!unlocked.workspace.show_mode() && unlocked.editable());
}

#[test]
fn an_added_cue_shows_the_basics_tab_but_a_refused_add_keeps_the_tab() {
    let (added, _) = on_levels(Some(CueId(4))).handled(Message::Add(Place::After, 1));
    assert_eq!(added.tab, Tab::Basics);
    let (refused, _) = locked(on_levels(Some(CueId(4)))).handled(Message::Add(Place::After, 1));
    assert_eq!(refused.tab, Tab::Levels);
}

#[test]
fn a_key_of_adding_or_hint_mode_takes_the_keyboard_from_a_list_that_has_it() {
    let units = |any, keys| {
        typing_task(
            App {
                keyboard: Keyboard {
                    held: Held {
                        any,
                        ..Held::default()
                    },
                    ..Keyboard::default()
                },
                ..show(Some(CueId(4)))
            },
            keys,
        )
        .1
        .units()
    };
    ["o", "O", "f"].into_iter().for_each(|keys| {
        assert!(units(true, keys) > units(false, keys), "{keys}");
    });
    // A key that acts in a list and leaves it where it is
    assert_eq!(units(true, "u"), units(false, "u"));
}
