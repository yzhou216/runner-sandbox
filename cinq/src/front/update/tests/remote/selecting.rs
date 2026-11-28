// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the playhead command and of a cue number that no cue has

use super::*;

#[test]
fn playhead_moves_standby_and_selects_the_cue_in_its_list() {
    let app = remote(playable(Some(CueId(2))), Remote::Playhead("21".into()));
    assert_eq!(app.workspace.lists()[1].standby, Some(CueId(21)));
    assert_eq!(app.selected, Some(CueId(21)));
    assert_eq!(app.list, 1);
    assert!(app.workspace.active().is_empty());
    let gone = after(app, [Message::Go]);
    assert_eq!(running(&gone), [21]);
}

#[test]
fn cue_number_that_no_cue_has_does_nothing() {
    let before = playable(Some(CueId(2)));
    for command in [
        Remote::Start("99".into()),
        Remote::Stop("99".into()),
        Remote::PauseCue("99".into()),
        Remote::ResumeCue("99".into()),
        Remote::Playhead("99".into()),
    ] {
        let app = remote(playable(Some(CueId(2))), command);
        assert!(app.workspace.active().is_empty());
        assert_eq!(app.selected, before.selected);
        assert_eq!(app.workspace.lists()[0].standby, Some(CueId(1)));
    }
}

#[test]
fn command_that_moves_the_selection_takes_the_keyboard_out_of_a_field() {
    let units = |app: App, message| app.next(message).1.units();
    let at = |id| playable(Some(CueId(id)));
    // The key of GO reveals the selection that it moves, and no key unfocuses
    assert_eq!(
        units(at(1), Message::Remote(FROM, Remote::Go)),
        units(at(1), Message::Go) + 1
    );
    // Selection stays, as it does when it is not standby
    assert_eq!(
        units(at(5), Message::Remote(FROM, Remote::Go)),
        units(at(5), Message::Go)
    );
    assert_eq!(units(at(5), Message::Remote(FROM, Remote::Pause)), 0);
    let playhead = |number: &str| Message::Remote(FROM, Remote::Playhead(number.to_owned()));
    assert_eq!(units(at(2), playhead("5")), units(at(2), playhead("2")) + 1);
}

#[test]
fn playhead_selects_as_a_click_does_closing_search_and_leaving_the_outputs() {
    let searching = super::search::opened(2);
    assert!(searching.search.is_some());
    let moved = remote(searching, Remote::Playhead("4".into()));
    assert!(moved.search.is_none());
    assert_eq!(moved.selected, Some(CueId(4)));
    let outputs = after(playable(Some(CueId(2))), [Message::ShowOutputs]);
    assert!(outputs.outputs_shown);
    let moved = remote(outputs, Remote::Playhead("5".into()));
    assert!(!moved.outputs_shown);
    assert_eq!(moved.selected, Some(CueId(5)));
    assert!(after(moved, [Message::Back(1)]).outputs_shown);
}

#[test]
fn cue_number_that_no_cue_has_leaves_typed_keys_be_and_hint_mode_only_for_a_new_line() {
    let (hinting, _) = playable(None).hinting();
    let first = remote(hinting, Remote::Start("none".into()));
    assert!(first.hints.is_none(), "the sidebar gains a line");
    let (again, _) = first.hinting();
    assert!(
        remote(again, Remote::Start("none".into())).hints.is_some(),
        "a line that was there moves nothing"
    );
    let halfway = typing(playable(None), "2");
    assert!(!halfway.keys.is_empty());
    let kept = remote(halfway, Remote::Stop("none".into()));
    assert!(!kept.keys.is_empty());
    let dropped = remote(typing(playable(None), "2"), Remote::Pause);
    assert!(dropped.keys.is_empty());
}

#[test]
fn command_that_moves_the_selection_closes_the_search_line_it_unfocuses() {
    let searching = |id| super::search::opening(playable(Some(CueId(id))), false);
    let moved = remote(searching(1), Remote::Go);
    assert_eq!(moved.selected, Some(CueId(2)));
    assert!(moved.search.is_none());
    // Selection that stays leaves the line open
    let kept = remote(searching(5), Remote::Go);
    assert_eq!(kept.selected, Some(CueId(5)));
    assert!(kept.search.is_some());
}
