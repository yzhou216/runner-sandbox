// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the hints that fit in the status bar, and of the room that the
//! search line keeps from them

mod cancel;

use super::{
    NO_MATCH, SEARCH_ROOM, clipped, columns, fitted, hints as hinted, mark, search_columns,
};
use crate::{
    back::{Command, Cue, CueId, CueList, Kind, Workspace},
    front::{
        App,
        keymap::{Action, Keymap, Preset, config::Config},
        update::Clipped,
    },
};

/// Returns the hints after a change, with unsaved changes
fn hints() -> Vec<(Option<Action>, String)> {
    [
        (None, "Added cue 1 | u: undo"),
        (Some(Action::Save), "Control+s: save"),
        (Some(Action::Go), "Space: GO"),
        (Some(Action::Panic), "Escape: Panic"),
        (Some(Action::Help), "?: help"),
    ]
    .into_iter()
    .map(|(action, hint)| (action, hint.to_owned()))
    .collect()
}

#[test]
fn hints_that_fit_show_in_full() {
    assert_eq!(
        fitted(&hints(), 77),
        "Added cue 1 | u: undo | Control+s: save | Space: GO | Escape: Panic | ?: help"
    );
}

#[test]
fn keys_give_way_in_turn_while_what_the_change_told_stays() {
    assert_eq!(
        fitted(&hints(), 76),
        "Added cue 1 | u: undo | Control+s: save | Escape: Panic | ?: help"
    );
    assert_eq!(fitted(&hints(), 31), "Added cue 1 | u: undo | ?: help");
    assert_eq!(fitted(&hints(), 5), "Added cue 1 | u: undo");
}

#[test]
fn keys_of_going_forward_and_back_give_way_before_the_rest() {
    let places = [
        (Some(Action::Back), "Control+o: back"),
        (Some(Action::Forward), "Control+i: forward"),
    ]
    .into_iter()
    .map(|(action, hint)| (action, hint.to_owned()));
    let hints: Vec<_> = hints()
        .into_iter()
        .take(1)
        .chain(places)
        .chain(hints().into_iter().skip(2))
        .collect();
    let all = "Added cue 1 | u: undo | Control+o: back | Control+i: forward | Space: GO | Escape: Panic | ?: help";
    assert_eq!(fitted(&hints, 100), all);
    assert_eq!(
        fitted(&hints, 80),
        "Added cue 1 | u: undo | Control+o: back | Space: GO | Escape: Panic | ?: help"
    );
    assert_eq!(
        fitted(&hints, 62),
        "Added cue 1 | u: undo | Space: GO | Escape: Panic | ?: help"
    );
}

#[test]
fn a_cut_names_the_keys_that_paste_and_undo_while_a_copy_names_only_the_paste_key() {
    let shown = |told: &str, cut: bool| App {
        clipped: Some(Clipped {
            told: told.to_owned(),
            cut,
        }),
        workspace: Workspace::new(vec![CueList::new(
            "Main",
            vec![Cue::new(CueId(1), "1", Kind::Memo, "Note").build()],
        )])
        .apply(Command::Delete(vec![CueId(1)])),
        ..App::default()
    };
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
fn what_a_key_remarked_shows_in_the_bar_before_anything_else() {
    let app = App {
        remark: Some("The clipboard is empty"),
        ..App::default()
    };
    let told = hinted(&app)
        .into_iter()
        .find_map(|(action, hint)| action.is_none().then_some(hint));
    assert_eq!(told.as_deref(), Some("The clipboard is empty"));
}

#[test]
fn hints_give_way_to_the_text_of_the_search_line_and_its_note() {
    assert_eq!(search_columns("", false), SEARCH_ROOM);
    assert_eq!(search_columns("storm", true), SEARCH_ROOM);
    assert_eq!(search_columns(&"x".repeat(40), false), 41);
    assert_eq!(
        search_columns(&"x".repeat(40), true),
        41 + NO_MATCH.len() + 1
    );
}

#[test]
fn the_search_line_starts_with_a_slash_or_a_question_mark_for_a_search_up() {
    assert_eq!((mark(false), mark(true)), ("/", "?"));
}

/// Returns the hints that an open menu adds, beside those of the keys of
/// GO and Panic
fn in_menu(app: App) -> Vec<String> {
    hinted(&app)
        .into_iter()
        .filter_map(|(action, hint)| {
            action
                .filter(|action| matches!(action, Action::Pick | Action::CloseMenu))
                .map(|_| hint)
        })
        .collect()
}

fn opened(preset: Preset, menu: bool, help: bool) -> App {
    let mut config = Config::default();
    config.keymap = Keymap::from(preset);
    App {
        config,
        menu,
        help,
        ..App::default()
    }
}

#[test]
fn an_open_menu_hints_the_keys_that_pick_and_close_it() {
    assert_eq!(
        in_menu(opened(Preset::Vi, true, false)),
        ["Control+y: pick", "Control+e: close"]
    );
    assert_eq!(
        in_menu(opened(Preset::Cua, true, false)),
        ["Enter: pick", "F4: close"]
    );
}

#[test]
fn a_closed_menu_and_a_menu_under_the_help_page_hint_nothing() {
    assert!(in_menu(opened(Preset::Vi, false, false)).is_empty());
    assert!(in_menu(opened(Preset::Vi, true, true)).is_empty());
}

#[test]
fn hints_of_a_menu_give_way_after_the_rest_when_the_bar_is_short() {
    let hints: Vec<_> = [
        (Some(Action::Pick), "Control+y: pick"),
        (Some(Action::CloseMenu), "Control+e: close"),
        (Some(Action::Go), "Space: GO"),
        (Some(Action::Help), "F1: help"),
    ]
    .into_iter()
    .map(|(action, hint)| (action, hint.to_owned()))
    .collect();
    assert_eq!(
        fitted(&hints, 45),
        "Control+y: pick | Control+e: close | F1: help"
    );
    assert_eq!(fitted(&hints, 34), "Control+y: pick | Control+e: close");
    assert_eq!(fitted(&hints, 20), "Control+y: pick");
}

#[test]
fn the_help_hint_beside_an_open_menu_reads_f1_in_the_vi_keymap() {
    let help = |preset| {
        hinted(&opened(preset, true, false))
            .into_iter()
            .find_map(|(action, hint)| (action == Some(Action::Help)).then_some(hint))
    };
    assert_eq!(help(Preset::Vi).as_deref(), Some("F1: help"));
    assert_eq!(help(Preset::Cua).as_deref(), Some("F1: help"));
}
