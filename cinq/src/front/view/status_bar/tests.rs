// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the hints that fit in the status bar, and of the room that the
//! search line keeps from them

mod adding;
mod cancel;
mod changes;
mod fields;
mod fixtures;
mod hint_mode;
mod lists;
mod menu;
mod name;
mod narrow;
mod next_list;
mod open;
mod outputs;
mod places;
mod stage;
mod standby;

use super::{
    GAP, SEARCH_ROOM,
    fit::{fitted, shown},
    hints::{hints as hinted, notices::clipped},
    mark,
    order::YIELDING,
    search_columns,
};
use crate::{
    back::{Command, Workspace, memo},
    front::{App, app::Notices, keymap::Action, view::NO_MATCH},
};
use fixtures::{
    GO, add, added, before, clipping, covered, deleted, has_hint, held, hints, hints_fitted,
    holding, keyed, opened, opened_with, owned, taken, texts,
};

#[test]
fn hints_that_fit_show_in_full_and_keys_give_way_in_turn_while_what_the_change_told_stays() {
    assert_eq!(
        fitted(&hints(), 77, &YIELDING),
        "Added cue 1 | u: undo | Control+s: save | Space: GO | Escape: Panic | ?: help"
    );
    assert_eq!(
        fitted(&hints(), 76, &YIELDING),
        "Added cue 1 | u: undo | Control+s: save | Escape: Panic | ?: help"
    );
    assert_eq!(
        fitted(&hints(), 31, &YIELDING),
        "Added cue 1 | u: undo | ?: help"
    );
    assert_eq!(fitted(&hints(), 5, &YIELDING), "Added cue 1 | u: undo");
}

#[test]
fn keys_of_going_forward_and_back_give_way_before_the_rest() {
    let places = owned([
        (Some(Action::Back), "Control+o: back"),
        (Some(Action::Forward), "Control+i: forward"),
    ]);
    let hints: Vec<_> = hints()
        .into_iter()
        .take(1)
        .chain(places)
        .chain(hints().into_iter().skip(2))
        .collect();
    let all = "Added cue 1 | u: undo | Control+o: back | Control+i: forward | Space: GO | Escape: Panic | ?: help";
    assert_eq!(fitted(&hints, 100, &YIELDING), all);
    assert_eq!(
        fitted(&hints, 80, &YIELDING),
        "Added cue 1 | u: undo | Control+o: back | Space: GO | Escape: Panic | ?: help"
    );
    assert_eq!(
        fitted(&hints, 62, &YIELDING),
        "Added cue 1 | u: undo | Space: GO | Escape: Panic | ?: help"
    );
}

#[test]
fn a_cut_names_the_keys_that_paste_and_undo_while_a_copy_names_only_the_paste_key() {
    let shown = |told: &str, cut: bool| clipping(told, cut, deleted());
    assert_eq!(
        clipped(&shown("Cut cue 4", true)).as_deref(),
        Some("Cut cue 4 | p: paste | u: undo")
    );
    assert_eq!(
        clipped(&shown("Copied 3 cues", false)).as_deref(),
        Some("Copied 3 cues | p: paste")
    );
    // Show mode locks pasting, so no key is offered
    let locked = App {
        workspace: Workspace::default().apply(Command::ShowMode(true)),
        ..shown("Copied 3 cues", false)
    };
    assert_eq!(clipped(&locked).as_deref(), Some("Copied 3 cues"));
}

#[test]
fn what_a_key_remarked_or_why_a_field_was_not_applied_shows_in_the_bar_before_anything_else() {
    let told = |app: App| {
        hinted(&app)
            .into_iter()
            .find_map(|(action, hint)| action.is_none().then_some(hint))
    };
    let remarked = App {
        notices: Notices {
            remark: Some("The clipboard is empty"),
            ..Notices::default()
        },
        ..App::default()
    };
    assert_eq!(told(remarked).as_deref(), Some("The clipboard is empty"));
    let unapplied = App {
        notices: Notices {
            unapplied: Some("Not applied: the name is empty".to_owned()),
            ..Notices::default()
        },
        ..App::default()
    };
    assert_eq!(
        told(unapplied).as_deref(),
        Some("Not applied: the name is empty")
    );
}

#[test]
fn hints_give_way_to_the_text_of_the_search_line_and_its_note() {
    assert_eq!(search_columns("", false, false), SEARCH_ROOM);
    assert_eq!(search_columns("storm", true, false), SEARCH_ROOM);
    assert_eq!(search_columns(&"x".repeat(40), false, false), 41);
    assert_eq!(
        search_columns(&"x".repeat(40), true, false),
        41 + NO_MATCH.len() + GAP
    );
}

#[test]
fn the_search_line_starts_with_a_slash_or_a_question_mark_for_a_search_up() {
    assert_eq!((mark(false), mark(true)), ("/", "?"));
}

#[test]
fn a_notice_that_does_not_fit_drops_the_key_that_teaches_then_gives_way_at_a_word() {
    let told = "Cut the output House L, so that cues play on House R | p: paste | u: undo";
    let shown = |room| shown(before([GO]), Some(told.to_owned()), room, &YIELDING, &[]);
    assert_eq!(shown(85), format!("{told} | Space: GO"));
    assert_eq!(shown(73), told, "the key gives way first");
    // The key that pastes gives way before the notice does, and the key that
    // takes the change back stays
    let lighter = "Cut the output House L, so that cues play on House R | u: undo";
    assert_eq!(shown(62), lighter);
    assert_eq!(
        shown(50),
        "Cut the output House L, so that cues… | u: undo",
        "the comma is not left before the ellipsis, nor half a word"
    );
    assert_eq!(
        shown(33),
        "Cut the output House L | u: undo",
        "a cut at a comma ends a clause that is whole"
    );
    assert_eq!(shown(29), "Cut the output… | u: undo");
    assert_eq!(
        shown(5),
        "Cut …",
        "with no word to cut at, it ends in an ellipsis at the edge"
    );
}

#[test]
fn a_notice_cut_where_a_name_ended_in_an_ellipsis_gets_no_second_one() {
    let told = "Cues play on Dante Virtual S…, deleted the output Sub | u: undo";
    assert_eq!(
        shown(before([]), Some(told.to_owned()), 45, &YIELDING, &[]),
        "Cues play on Dante Virtual S… | u: undo"
    );
}
