// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of searching the cues by name: matching, the row that a search
//! starts from, wrapping, searching up the list, cancelling by Backspace and
//! by Escape, no match, and the keys of GO and Panic, which still act over
//! the search line

use iced::keyboard::{Key, key::Named};

use super::press;
use crate::{
    back::{Cue, CueId, CueList, Kind, Workspace},
    front::{App, Message, view},
};

/// Returns the state of the interface with `selected` selected in a list of
/// five cues
pub(super) fn show(selected: Option<CueId>) -> App {
    let cue =
        |id: u32, number: &str, name: &str| Cue::new(CueId(id), number, Kind::Memo, name).build();
    let cues = vec![
        cue(1, "1", "Storm opening"),
        cue(2, "2", "Rain loop"),
        cue(3, "3", "Thunder"),
        cue(4, "4", "Storm end"),
        cue(5, "A5", "Wind"),
    ];
    App {
        workspace: Workspace::new(vec![CueList::new("Main", cues)]),
        selected,
        ..App::default()
    }
}

/// Opens the search line with `selected` selected
pub(super) fn opened(selected: u32) -> App {
    show(Some(CueId(selected)))
        .handled(Message::Search { backward: false })
        .0
}

/// Opens the line of a search up the list
pub(super) fn opened_backward(selected: u32) -> App {
    show(Some(CueId(selected)))
        .handled(Message::Search { backward: true })
        .0
}

pub(super) fn typed(app: App, text: &str) -> App {
    app.handled(Message::Searching(text.to_owned())).0
}

fn warned(app: &App) -> bool {
    app.search.as_ref().is_some_and(|search| search.warned)
}

pub(super) fn searched(app: App, text: &str) -> App {
    typed(app, text).handled(Message::SearchAccepted).0
}

#[test]
fn search_selects_the_first_cue_below_the_selection_that_holds_the_text() {
    assert_eq!(typed(opened(2), "storm").selected, Some(CueId(4)));
    // Number counts, and wrapping goes back to the top
    assert_eq!(typed(opened(1), "a5").selected, Some(CueId(5)));
    assert_eq!(typed(opened(5), "storm").selected, Some(CueId(1)));
    assert_eq!(typed(opened(5), "ra").selected, Some(CueId(2)));
}

#[test]
fn search_goes_on_past_the_selection_and_ends_on_it_last_as_in_vim() {
    // Cue 4 holds the text, and the search wraps around to the other cue
    assert_eq!(typed(opened(4), "STORM").selected, Some(CueId(1)));
    assert_eq!(typed(opened(1), "storm").selected, Some(CueId(4)));
    // The selection alone holds it, so the search comes back to it
    assert_eq!(typed(opened(5), "wind").selected, Some(CueId(5)));
}

#[test]
fn search_up_the_list_selects_the_first_cue_above_the_selection_that_holds_the_text() {
    assert_eq!(typed(opened_backward(4), "storm").selected, Some(CueId(1)));
    assert_eq!(typed(opened_backward(1), "storm").selected, Some(CueId(4)));
    assert_eq!(typed(opened_backward(3), "o").selected, Some(CueId(2)));
    assert_eq!(typed(opened_backward(5), "wind").selected, Some(CueId(5)));
    assert!(
        opened_backward(2)
            .search
            .is_some_and(|search| search.backward)
    );
}

#[test]
fn search_looks_from_where_it_began_with_each_key_typed() {
    let typing = typed(opened(2), "t");
    assert_eq!(typing.selected, Some(CueId(3)));
    let typing = typed(typing, "th");
    assert_eq!(typing.selected, Some(CueId(3)));
    // Deleting a character searches again from the start, not from the match
    let typing = typed(typing, "t");
    assert_eq!(typing.selected, Some(CueId(3)));
    let typing = typed(typing, "");
    assert_eq!(typing.selected, Some(CueId(2)));
    assert!(typing.search.is_some());
}

#[test]
fn enter_accepts_the_search_and_closes_the_line() {
    let (accepted, _) = typed(opened(2), "storm").handled(Message::SearchAccepted);
    assert!(accepted.search.is_none());
    assert_eq!(
        (accepted.selected, accepted.searched.text.as_str()),
        (Some(CueId(4)), "storm")
    );
    assert!(!accepted.searched.backward);
    let (backward, _) = typed(opened_backward(2), "storm").handled(Message::SearchAccepted);
    assert!(backward.searched.backward);
}

#[test]
fn backspace_on_an_empty_line_closes_it_and_gives_the_selection_back() {
    let typing = typed(opened(2), "w");
    assert_eq!(typing.selected, Some(CueId(5)));
    let typing = typed(typing, "");
    let (closed, _) = typing.handled(Message::Searching(String::new()));
    assert!(closed.search.is_none());
    assert_eq!(closed.selected, Some(CueId(2)));
    // Nothing was accepted
    assert_eq!(closed.searched.text, "");
}

#[test]
fn escape_closes_the_line_and_gives_the_selection_back() {
    let typing = typed(opened(2), "wind");
    assert_eq!(typing.selected, Some(CueId(5)));
    let (closed, _) = typing.typed(press(Key::Named(Named::Escape)), false);
    assert!(closed.search.is_none());
    assert_eq!(closed.selected, Some(CueId(2)));
    assert_eq!(closed.searched.text, "");
}

#[test]
fn escape_without_a_search_leaves_the_selection_alone() {
    let (panicked, _) = show(Some(CueId(3))).typed(press(Key::Named(Named::Escape)), false);
    assert_eq!(panicked.selected, Some(CueId(3)));
    let (panicked, _) = show(None).typed(press(Key::Named(Named::Escape)), false);
    assert_eq!(panicked.selected, None);
}

#[test]
fn search_with_no_match_leaves_the_selection_where_it_began_and_tells_so() {
    let typing = typed(opened(2), "w");
    assert_eq!(typing.selected, Some(CueId(5)));
    let missed = typed(typing, "wz");
    assert_eq!(missed.selected, Some(CueId(2)));
    assert!(warned(&missed));
    let (settled, _) = missed.handled(Message::SearchSettled);
    assert!(!warned(&settled));
    assert!(settled.search.is_some());
    // Enter keeps the line open to tell so again
    let (entered, _) = settled.handled(Message::SearchAccepted);
    assert!(warned(&entered));
    assert_eq!(
        (entered.selected, entered.searched.text.as_str()),
        (Some(CueId(2)), "")
    );
    // Without a selection, the selection stays empty
    assert_eq!(
        typed(
            typed(
                show(None).handled(Message::Search { backward: false }).0,
                "w"
            ),
            "wz"
        )
        .selected,
        None
    );
}

#[test]
fn space_and_escape_still_act_with_the_search_line_open() {
    let standby = |app: &App| app.workspace.lists()[0].standby;
    let before = standby(&opened(2));
    let (gone, _) = opened(2).typed(press(Key::Named(Named::Space)), false);
    assert_ne!(standby(&gone), before);
    let (panicked, _) = opened(2).typed(press(Key::Named(Named::Escape)), false);
    assert!(panicked.search.is_none());
}

#[test]
fn window_draws_with_the_search_line_open_and_with_a_note() {
    let _ = view::view(&typed(opened(2), "storm"));
    let _ = view::view(&typed(opened_backward(2), "storm"));
    let _ = view::view(&typed(opened(2), "zz"));
}
