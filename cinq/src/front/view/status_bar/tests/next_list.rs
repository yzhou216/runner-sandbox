// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the hint of the key that shows the next cue list

use super::{super::order::yielding, YIELDING, covered, has_hint, hints_fitted, memo, opened_with};
use crate::{
    back::{CueList, Workspace},
    front::{
        App,
        keymap::{Keymap, desk},
    },
};

/// Returns an app of two lists, the first of which has a standby cue unless
/// `ended`, when it holds no cue and standby lies past its end
fn two_lists(keymap: Keymap, ended: bool) -> App {
    let first = if ended { vec![] } else { vec![memo(1)] };
    App {
        workspace: Workspace::new(vec![
            CueList::new("Act 1", first),
            CueList::new("Act 2", vec![memo(2)]),
        ]),
        ..opened_with(keymap, false, false)
    }
}

#[test]
fn the_key_of_the_next_list_is_hinted_whatever_the_keymap_binds() {
    let told = |keymap| hints_fitted(&two_lists(keymap, false), 120);
    assert!(told(Keymap::default()).ends_with("l: next list | f: hint"));
    let long = desk("next_list = \"Control+PageDown\"");
    assert!(told(long).ends_with("Control+PageDown: next list | f: hint"));
}

#[test]
fn the_hint_gives_way_after_the_next_list_and_the_next_tab_in_a_list_with_a_standby_cue() {
    let app = two_lists(Keymap::default(), false);
    let all = hints_fitted(&app, 100);
    assert_eq!(
        all,
        "Space: GO | Escape: Panic | ?: help | gt: next tab | l: next list | f: hint"
    );
    assert_eq!(
        hints_fitted(&app, all.len() - 1),
        "Space: GO | Escape: Panic | ?: help | gt: next tab | f: hint"
    );
    // The keys of the next tab and the next list, which only teach, go first
    assert_eq!(
        hints_fitted(&app, all.len() - 16),
        "Space: GO | Escape: Panic | ?: help | f: hint"
    );
    assert_eq!(yielding(&app), YIELDING);
}

#[test]
fn the_hint_outlasts_the_hint_of_go_where_standby_is_past_the_end_and_a_list_follows() {
    let app = two_lists(Keymap::default(), true);
    let all = hints_fitted(&app, 100);
    assert_eq!(
        all,
        "Space: GO | Escape: Panic | ?: help | gt: next tab | l: next list | f: hint"
    );
    // Tab and the rest give way first, then GO, which fires nothing there, and
    // Panic, while the key of the next list outlasts the key of help
    assert_eq!(
        hints_fitted(&app, 41),
        "Escape: Panic | ?: help | l: next list"
    );
    assert_eq!(hints_fitted(&app, 37), "?: help | l: next list");
    assert_eq!(hints_fitted(&app, 21), "l: next list");
}

#[test]
fn the_last_list_has_the_outputs_to_go_to_and_gives_the_hint_no_more_room() {
    let last = App {
        list: 1,
        ..two_lists(Keymap::default(), true)
    };
    assert_eq!(yielding(&last), YIELDING);
}

#[test]
fn no_field_view_or_page_that_has_the_keys_hints_the_next_list() {
    let lists = || two_lists(Keymap::default(), true);
    assert!(has_hint(&lists(), "l: next list"));
    assert!(
        covered(lists)
            .iter()
            .all(|app| !has_hint(app, "l: next list"))
    );
}
