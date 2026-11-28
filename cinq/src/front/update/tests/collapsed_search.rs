// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the groups that the preview of a search expands, which close
//! again when the match moves elsewhere or the search is cancelled

use super::{
    collapsed::{closed, collapsed, folded, selected},
    search::{opening, searched, typed},
    *,
};
use crate::front::message::Fold;

/// Returns the show of `collapsed` with both groups closed, and the search
/// line open from the cue `from`
fn closing(from: u32, backward: bool) -> App {
    opening(folded(collapsed(from), Fold::CloseAll), backward)
}

#[test]
fn cancelled_search_closes_what_it_opened() {
    let found = typed(closing(7, true), "gust");
    assert_eq!(selected(&found), Some(5));
    assert!(!closed(&found, 2) && !closed(&found, 4));
    let cancelled = found.search_cancelled();
    assert_eq!(selected(&cancelled), Some(7));
    assert!(closed(&cancelled, 2) && closed(&cancelled, 4));
}

#[test]
fn control_c_and_escape_close_what_the_preview_opened() {
    let found = typed(closing(7, true), "gust");
    let (control_c, _) = typed(closing(7, true), "gust").handled(Message::Cancel);
    let (escape, _) = found.handled(Message::Panic);
    for cancelled in [control_c, escape] {
        assert!(cancelled.search.is_none());
        assert_eq!(selected(&cancelled), Some(7));
        assert!(closed(&cancelled, 2) && closed(&cancelled, 4));
    }
}

#[test]
fn only_the_groups_that_hide_the_match_stay_open_while_typing() {
    let deep = typed(closing(1, false), "gust");
    assert!(!closed(&deep, 2) && !closed(&deep, 4));
    let shallow = typed(deep, "thunder");
    assert_eq!(selected(&shallow), Some(3));
    assert!(!closed(&shallow, 2) && closed(&shallow, 4));
    let outside = typed(shallow, "birds");
    assert_eq!(selected(&outside), Some(7));
    assert!(closed(&outside, 2) && closed(&outside, 4));
    let missed = typed(typed(outside, "gust"), "nothing like this");
    assert_eq!(selected(&missed), Some(1));
    assert!(closed(&missed, 2) && closed(&missed, 4));
}

#[test]
fn a_group_that_was_open_before_the_search_stays_open() {
    let open = opening(folded(collapsed(1), Fold::OpenAll), false);
    let found = typed(open, "gust");
    assert_eq!(selected(&found), Some(5));
    let cancelled = found.search_cancelled();
    assert!(!closed(&cancelled, 2) && !closed(&cancelled, 4));
}

#[test]
fn accepting_the_search_keeps_the_groups_of_the_match_open() {
    let accepted = searched(closing(1, false), "gust");
    assert_eq!(selected(&accepted), Some(5));
    assert!(!closed(&accepted, 2) && !closed(&accepted, 4));
}

#[test]
fn the_keys_of_the_next_match_keep_the_groups_they_open() {
    let accepted = searched(closing(1, false), "r");
    assert_eq!(selected(&accepted), Some(2));
    let next = accepted.handled(Message::SearchStep(1)).0;
    assert_eq!(selected(&next), Some(3));
    let again = next.handled(Message::SearchStep(1)).0;
    assert!(!closed(&again, 2));
    let back = again.handled(Message::SearchStep(-1)).0;
    assert!(!closed(&back, 2));
}
