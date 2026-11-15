// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys of the next and previous match: the direction of the
//! search that they follow, wrapping, and counts of any size

use iced::keyboard::Key;

use super::{
    press,
    search::{opened, opened_backward, searched, show, typed},
};
use crate::{
    back::CueId,
    front::{App, Message, update::Searched},
};

fn stepped(app: App, offset: isize) -> App {
    app.handled(Message::SearchStep(offset)).0
}

#[test]
fn next_and_previous_match_wrap_around_the_list() {
    let accepted = searched(opened(3), "storm");
    assert_eq!(accepted.selected, Some(CueId(4)));
    let next = stepped(accepted, 1);
    assert_eq!(next.selected, Some(CueId(1)));
    let next = stepped(next, 1);
    assert_eq!(next.selected, Some(CueId(4)));
    let previous = stepped(next, -1);
    assert_eq!(previous.selected, Some(CueId(1)));
    let previous = stepped(previous, -1);
    assert_eq!(previous.selected, Some(CueId(4)));
    // Count repeats the step
    assert_eq!(stepped(previous, 3).selected, Some(CueId(1)));
    assert_eq!(
        stepped(stepped(searched(opened(2), "o"), -1), -2).selected,
        Some(CueId(4))
    );
}

#[test]
fn the_next_match_follows_the_direction_of_the_search_and_the_previous_goes_against_it() {
    // Matches of o are cues 1 and 2, as cue 4 holds none
    let accepted = searched(opened_backward(5), "storm");
    assert_eq!(accepted.selected, Some(CueId(4)));
    let next = stepped(accepted, 1);
    assert_eq!(next.selected, Some(CueId(1)));
    let next = stepped(next, 1);
    assert_eq!(next.selected, Some(CueId(4)));
    let previous = stepped(next, -1);
    assert_eq!(previous.selected, Some(CueId(1)));
    let previous = stepped(previous, -1);
    assert_eq!(previous.selected, Some(CueId(4)));
    assert_eq!(stepped(previous, 3).selected, Some(CueId(1)));
}

#[test]
fn the_keys_of_next_and_previous_match_name_the_direction_in_a_search_up_the_list() {
    let keys = |app: App, keys: &[&str]| {
        keys.iter().fold(app, |app, &key| {
            app.typed(press(Key::Character(key.into())), false).0
        })
    };
    // Cues 1, 2, and 4 hold an o
    let up = || searched(opened_backward(4), "o");
    assert_eq!(up().selected, Some(CueId(2)));
    assert_eq!(keys(up(), &["n"]).selected, Some(CueId(1)));
    assert_eq!(keys(up(), &["N"]).selected, Some(CueId(4)));
    assert_eq!(keys(up(), &["2", "n"]).selected, Some(CueId(4)));
    assert!(
        keys(up(), &["?"])
            .search
            .is_some_and(|search| search.backward)
    );
}

#[test]
fn next_and_previous_match_do_nothing_before_a_search() {
    let stepped = |offset| stepped(show(Some(CueId(3))), offset);
    assert_eq!(
        (stepped(1).selected, stepped(-1).selected),
        (Some(CueId(3)), Some(CueId(3)))
    );
    assert_eq!(show(None).handled(Message::SearchStep(1)).0.selected, None);
}

#[test]
fn next_and_previous_match_keys_step_with_the_count() {
    let accepted = searched(opened(1), "o");
    // Matches are cues 1, 2, and 4, which hold an o
    let keys = |app: App, keys: &[&str]| {
        keys.iter().fold(app, |app, &key| {
            app.typed(press(Key::Character(key.into())), false).0
        })
    };
    assert_eq!(accepted.selected, Some(CueId(2)));
    assert_eq!(keys(accepted, &["n"]).selected, Some(CueId(4)));
    assert_eq!(
        keys(searched(opened(1), "o"), &["2", "n"]).selected,
        Some(CueId(1))
    );
    assert_eq!(
        keys(searched(opened(1), "o"), &["N"]).selected,
        Some(CueId(1))
    );
    assert!(
        keys(searched(opened(1), "o"), &["/"])
            .search
            .is_some_and(|search| !search.backward)
    );
}

#[test]
fn empty_line_accepted_repeats_the_last_search_in_the_direction_of_the_line() {
    let accepted = searched(opened(1), "storm");
    let (again, _) = accepted
        .handled(Message::Search { backward: false })
        .0
        .handled(Message::SearchAccepted);
    assert!(again.search.is_none());
    assert_eq!(again.selected, Some(CueId(1)));
    let (up, _) = searched(opened(1), "storm")
        .handled(Message::Search { backward: true })
        .0
        .handled(Message::SearchAccepted);
    assert_eq!(up.selected, Some(CueId(1)));
    assert!(up.searched.backward);
}

#[test]
fn a_huge_count_steps_at_most_one_lap() {
    // Two cues hold storm, so an even count comes back and an odd one moves
    let stepped = |offset| stepped(searched(opened(3), "storm"), offset).selected;
    assert_eq!(stepped(99_999_999), Some(CueId(1)));
    assert_eq!(stepped(100_000_000), Some(CueId(4)));
    assert_eq!(stepped(isize::MAX), Some(CueId(1)));
    assert_eq!(stepped(isize::MIN), Some(CueId(4)));
    // Count equal to the matches comes back to the same cue, as in Vim
    assert_eq!(stepped(2), Some(CueId(4)));
    assert_eq!(stepped(-2), Some(CueId(4)));
    assert_eq!(stepped(3), Some(CueId(1)));
}

#[test]
fn a_huge_count_of_the_key_of_next_match_returns() {
    let keys = ["9", "9", "9", "9", "9", "9", "9", "9", "n"]
        .into_iter()
        .fold(searched(opened(3), "storm"), |app, key| {
            app.typed(press(Key::Character(key.into())), false).0
        });
    assert_eq!(keys.selected, Some(CueId(1)));
}

#[test]
fn stepping_with_no_matching_cue_does_nothing_at_any_count() {
    let missing = App {
        searched: Searched {
            text: "zz".to_owned(),
            backward: false,
            ..Searched::default()
        },
        ..show(Some(CueId(3)))
    };
    assert_eq!(stepped(missing, isize::MAX).selected, Some(CueId(3)));
}

const TOP: Option<&str> = Some("Search continued at the top");
const BOTTOM: Option<&str> = Some("Search continued at the bottom");

#[test]
fn a_search_that_goes_past_the_end_of_the_list_tells_where_it_continued() {
    // Cues 1 and 4 hold Storm, so that a search from 4 goes round to 1
    assert_eq!(typed(opened(4), "storm").remark, TOP);
    assert_eq!(searched(opened(4), "storm").remark, TOP);
    assert_eq!(searched(opened(1), "storm").remark, None);
    assert_eq!(typed(opened_backward(1), "storm").remark, BOTTOM);
    assert_eq!(searched(opened_backward(1), "storm").remark, BOTTOM);
    assert_eq!(searched(opened_backward(4), "storm").remark, None);
}

#[test]
fn the_keys_of_the_next_and_previous_match_tell_where_they_continued_too() {
    let accepted = searched(opened(1), "storm");
    assert_eq!(accepted.remark, None);
    let next = stepped(accepted, 1);
    assert_eq!((next.selected, next.remark), (Some(CueId(1)), TOP));
    let previous = stepped(next, -1);
    assert_eq!(
        (previous.selected, previous.remark),
        (Some(CueId(4)), BOTTOM)
    );
    // Step that stays within the list tells nothing, and takes the remark away
    assert_eq!(stepped(previous, -1).remark, None);
    // Count that goes round tells it
    assert_eq!(stepped(searched(opened(1), "storm"), 3).remark, TOP);
}

#[test]
fn the_next_match_with_no_match_says_no_cue_matches_as_typing_does() {
    let none = searched(opened(2), "storm");
    let lost = App {
        searched: Searched {
            text: "zzz".to_owned(),
            backward: false,
            ..Searched::default()
        },
        ..none
    };
    assert_eq!(stepped(lost, 1).remark, Some("No cue matches"));
    // Before any search too
    assert_eq!(
        stepped(show(Some(CueId(2))), 1).remark,
        Some("No cue matches")
    );
}
