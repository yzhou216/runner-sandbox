// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that enter the name of the selected cue, at its end,
//! at its start, or with all of it selected

use super::{locked, on_levels, show, typing, typing_task};
use crate::{back::CueId, front::Tab};

#[test]
fn keys_of_vim_enter_the_name_of_the_selected_cue_and_show_its_tab() {
    ["i", "a", "A", "I", "S", "c c"]
        .into_iter()
        .for_each(|keys| {
            let (entered, task) = typing_task(on_levels(Some(CueId(3))), keys);
            assert_eq!(
                (entered.tab, task.units() > 0),
                (Tab::Basics, true),
                "{keys}"
            );
        });
}

#[test]
fn c_begins_no_operator_so_a_c_before_another_key_drops_both() {
    let moved = typing(on_levels(Some(CueId(3))), "c j");
    assert_eq!((moved.selected, moved.tab), (Some(CueId(3)), Tab::Levels));
    assert_eq!(moved.workspace.lists()[0].cues.len(), 12);
}

#[test]
fn entering_the_name_does_nothing_without_a_selection_or_in_show_mode() {
    let (none, task) = typing_task(show(None), "I");
    assert_eq!((none.tab, task.units()), (Tab::Basics, 0));
    let (locked, task) = typing_task(locked(on_levels(Some(CueId(3)))), "S");
    assert_eq!((locked.tab, task.units()), (Tab::Levels, 0));
}

#[test]
fn caret_at_the_start_or_over_all_the_name_takes_a_task_of_its_own() {
    let units = |key| typing_task(show(Some(CueId(3))), key).1.units();
    assert_eq!(units("A"), units("i"));
    assert!(units("I") > units("i"), "{} {}", units("I"), units("i"));
    assert!(units("S") > units("i"));
}
