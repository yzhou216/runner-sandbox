// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of searching the outputs by name

use super::{super::after, at, cue_list_kept, outputs};
use crate::front::{App, Message, update::search::Searched};

/// Returns the messages that open the search line and type `text` into it
fn searching(text: &str) -> [Message; 2] {
    [
        Message::Search { backward: false },
        Message::Searching(text.to_owned()),
    ]
}

#[test]
fn a_search_selects_the_outputs_by_name_as_it_selects_cues() {
    let search = |text: &str| {
        let app = after(outputs(), searching(text));
        (at(&app), app)
    };
    let (row, app) = search("sub");
    assert_eq!(row, 3);
    assert!(
        app.search.is_some(),
        "the line stays open while it is typed"
    );
    // First output after the selection that matches, in any case
    assert_eq!(search("HOUSE").0, 2);
    assert_eq!(search("o").0, 2);
    // No output matches, so that the selection stays where the search began
    let (row, missed) = search("zzz");
    assert_eq!(row, 1);
    assert!(missed.search.as_ref().is_some_and(|search| search.warned));
    cue_list_kept(&missed);
}

#[test]
fn accepting_a_search_in_the_outputs_is_a_jump_and_next_and_previous_match_repeat_it() {
    let accepted = after(
        outputs(),
        searching("r").into_iter().chain([Message::SearchAccepted]),
    );
    assert!(accepted.search.is_none());
    assert_eq!(at(&accepted), 2);
    // Rows holding an r, after House R: Monitor, as Sub has none
    let next = after(accepted, [Message::SearchStep(1)]);
    assert_eq!(at(&next), 4);
    let previous = after(next, [Message::SearchStep(-1)]);
    assert_eq!(at(&previous), 2);
    cue_list_kept(&previous);
    // Accepted search and the steps are jumps for going back
    let back = after(previous, [Message::Back(1)]);
    assert!(back.outputs_shown);
    assert_eq!(at(&back), 4);
}

#[test]
fn a_search_that_wraps_in_the_outputs_says_so_and_one_without_a_match_says_that() {
    let last = after(outputs(), [Message::Row(usize::MAX)]);
    let wrapped = after(
        last,
        searching("house")
            .into_iter()
            .chain([Message::SearchAccepted]),
    );
    assert_eq!(at(&wrapped), 1);
    assert_eq!(wrapped.notices.remark, Some("Search continued at the top"));
    let lost = App {
        searched: Searched {
            text: "zzz".to_owned(),
            ..Searched::default()
        },
        ..outputs()
    };
    let nothing = after(lost, [Message::SearchStep(1)]);
    assert_eq!(nothing.notices.remark, Some("No output matches"));
    assert_eq!(at(&nothing), 1);
}

#[test]
fn cancelling_a_search_in_the_outputs_puts_the_selection_back() {
    let app = after(
        outputs(),
        searching("booth").into_iter().chain([Message::Panic]),
    );
    assert!(app.search.is_none());
    assert_eq!(at(&app), 1);
}

#[test]
fn showing_the_cue_list_again_closes_the_search_line_of_the_outputs() {
    let app = after(
        outputs(),
        searching("sub").into_iter().chain([Message::SwitchList(0)]),
    );
    assert!(app.search.is_none());
    assert!(!app.outputs_shown);
}
