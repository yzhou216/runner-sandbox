// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that enter the name of the selected cue, at its end,
//! at its start, or with all of it selected

use iced::keyboard::Key;

use super::{press, show};
use crate::{
    back::CueId,
    front::{App, Message, Tab},
};

/// Types each of `keys` alone and returns the interface state with the
/// tasks that the last of them makes
fn typed(app: App, keys: &[&str]) -> (App, usize) {
    keys.iter().fold((app, 0), |(app, _), key| {
        let (app, task) = app.typed(press(Key::Character((*key).into())), false);
        (app, task.units())
    })
}

#[test]
fn keys_of_vim_enter_the_name_of_the_selected_cue_and_show_its_tab() {
    let tabbed = || App {
        tab: Tab::Levels,
        ..show(Some(CueId(3)))
    };
    [&["i"][..], &["a"], &["A"], &["I"], &["S"], &["c", "c"]]
        .iter()
        .for_each(|keys| {
            let (entered, units) = typed(tabbed(), keys);
            assert_eq!((entered.tab, units > 0), (Tab::Basics, true), "{keys:?}");
        });
}

#[test]
fn c_begins_no_operator_so_a_c_before_another_key_drops_both() {
    let levels = App {
        tab: Tab::Levels,
        ..show(Some(CueId(3)))
    };
    let (moved, _) = typed(levels, &["c", "j"]);
    assert_eq!((moved.selected, moved.tab), (Some(CueId(3)), Tab::Levels));
    assert_eq!(moved.workspace.lists()[0].cues.len(), 12);
}

#[test]
fn entering_the_name_does_nothing_without_a_selection_or_in_show_mode() {
    let (none, units) = typed(show(None), &["I"]);
    assert_eq!((none.tab, units), (Tab::Basics, 0));
    let (locked, _) = show(Some(CueId(3))).handled(Message::ShowMode(true));
    let locked = App {
        tab: Tab::Levels,
        ..locked
    };
    let (locked, units) = typed(locked, &["S"]);
    assert_eq!((locked.tab, units), (Tab::Levels, 0));
}

#[test]
fn caret_at_the_start_or_over_all_the_name_takes_a_task_of_its_own() {
    let units = |key| typed(show(Some(CueId(3))), &[key]).1;
    assert_eq!(units("A"), units("i"));
    assert!(units("I") > units("i"), "{} {}", units("I"), units("i"));
    assert!(units("S") > units("i"));
}
