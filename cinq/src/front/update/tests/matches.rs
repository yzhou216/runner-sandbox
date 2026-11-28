// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys of the next and previous match: the direction of the
//! search that they follow, wrapping, and counts of any size

use super::{
    search::{opened, opened_backward, opening, searched, show, typed},
    typing,
};
use crate::{
    back::CueId,
    front::{App, Message, update::Searched},
};

fn stepped(app: App, offset: isize) -> App {
    app.handled(Message::SearchStep(offset)).0
}

/// Returns the identity of the selected cue after each of the steps from the
/// state
fn stepping(app: App, offsets: impl IntoIterator<Item = isize>) -> Vec<u32> {
    offsets
        .into_iter()
        .scan(Some(app), |state, offset| {
            let next = stepped(state.take()?, offset);
            let selected = next.selected?.0;
            *state = Some(next);
            Some(selected)
        })
        .collect()
}

#[test]
fn next_and_previous_match_wrap_around_the_list() {
    let accepted = searched(opened(3), "storm");
    assert_eq!(accepted.selected, Some(CueId(4)));
    // The last step has a count, which repeats it
    assert_eq!(stepping(accepted, [1, 1, -1, -1, 3]), [1, 4, 1, 4, 1]);
    assert_eq!(stepping(searched(opened(2), "o"), [-1, -2]), [2, 4]);
}

#[test]
fn the_next_match_follows_the_direction_of_the_search_and_the_previous_goes_against_it() {
    // Cues 1, 2, and 4 hold an o, so that going up from 5 reads 4, 2, 1, and
    // the wrap to 4, while going down from 4 reaches 1 by the wrap
    let accepted = searched(opened_backward(5), "o");
    assert_eq!(accepted.selected, Some(CueId(4)));
    assert_eq!(stepping(accepted, [1, 1, 1, -1, -1, 2]), [2, 1, 4, 1, 2, 4]);
}

#[test]
fn the_keys_of_next_and_previous_match_name_the_direction_in_a_search_up_the_list() {
    // Cues 1, 2, and 4 hold an o
    let up = || searched(opened_backward(4), "o");
    assert_eq!(up().selected, Some(CueId(2)));
    assert_eq!(typing(up(), "n").selected, Some(CueId(1)));
    assert_eq!(typing(up(), "N").selected, Some(CueId(4)));
    assert_eq!(typing(up(), "2 n").selected, Some(CueId(4)));
    // The slash opens a line for a search down the list, and the question
    // mark shows the help page, which leaves the direction alone
    assert!(
        typing(up(), "/")
            .search
            .is_some_and(|search| !search.backward)
    );
    let help = typing(up(), "?");
    assert!(help.help && help.search.is_none() && help.searched.backward);
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
    // Matches are cues 1, 2, and 4, which hold an o
    let accepted = || searched(opened(1), "o");
    assert_eq!(accepted().selected, Some(CueId(2)));
    assert_eq!(typing(accepted(), "n").selected, Some(CueId(4)));
    assert_eq!(typing(accepted(), "2 n").selected, Some(CueId(1)));
    assert_eq!(typing(accepted(), "N").selected, Some(CueId(1)));
    assert!(typing(accepted(), "/").search.is_some_and(|s| !s.backward));
}

#[test]
fn empty_line_accepted_repeats_the_last_search_in_the_direction_of_the_line() {
    let accepted = searched(opened(1), "storm");
    let (again, _) = opening(accepted, false).handled(Message::SearchAccepted);
    assert!(again.search.is_none());
    assert_eq!(again.selected, Some(CueId(1)));
    let (up, _) = opening(searched(opened(1), "storm"), true).handled(Message::SearchAccepted);
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
    // The key takes the count typed digit by digit
    let typed = typing(searched(opened(3), "storm"), "9 9 9 9 9 9 9 9 n");
    assert_eq!(typed.selected, Some(CueId(1)));
}

/// Returns the state with an accepted search for `text` that no cue holds
fn lost(app: App, text: &str) -> App {
    App {
        searched: Searched {
            text: text.to_owned(),
            ..Searched::default()
        },
        ..app
    }
}

#[test]
fn stepping_with_no_matching_cue_does_nothing_at_any_count() {
    let missing = lost(show(Some(CueId(3))), "zz");
    assert_eq!(stepped(missing, isize::MAX).selected, Some(CueId(3)));
}

const TOP: Option<&str> = Some("Search continued at the top");
const BOTTOM: Option<&str> = Some("Search continued at the bottom");

#[test]
fn a_search_that_goes_past_the_end_of_the_list_tells_where_it_continued() {
    // Cues 1 and 4 hold Storm, so that a search from 4 goes round to 1
    assert_eq!(typed(opened(4), "storm").notices.remark, TOP);
    assert_eq!(searched(opened(4), "storm").notices.remark, TOP);
    assert_eq!(searched(opened(1), "storm").notices.remark, None);
    assert_eq!(typed(opened_backward(1), "storm").notices.remark, BOTTOM);
    assert_eq!(searched(opened_backward(1), "storm").notices.remark, BOTTOM);
    assert_eq!(searched(opened_backward(4), "storm").notices.remark, None);
}

#[test]
fn the_keys_of_the_next_and_previous_match_tell_where_they_continued_too() {
    let accepted = searched(opened(1), "storm");
    assert_eq!(accepted.notices.remark, None);
    let next = stepped(accepted, 1);
    assert_eq!((next.selected, next.notices.remark), (Some(CueId(1)), TOP));
    let previous = stepped(next, -1);
    assert_eq!(
        (previous.selected, previous.notices.remark),
        (Some(CueId(4)), BOTTOM)
    );
    // Step that stays within the list tells nothing, and takes the remark away
    assert_eq!(stepped(previous, -1).notices.remark, None);
    // Count that goes round tells it
    assert_eq!(stepped(searched(opened(1), "storm"), 3).notices.remark, TOP);
}

#[test]
fn the_next_match_with_no_match_says_no_cue_matches_as_typing_does() {
    let lost = lost(searched(opened(2), "storm"), "zzz");
    assert_eq!(stepped(lost, 1).notices.remark, Some("No cue matches"));
    // Before any search it says that there is none
    assert_eq!(
        stepped(show(Some(CueId(2))), 1).notices.remark,
        Some("No previous search")
    );
}
