// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the hints that an open menu adds, and of how they give way

use super::{
    super::{fit::fitted, hints::hints as hinted, order::YIELDING},
    opened, opened_with, owned,
};
use crate::{
    back::{Command, Kind, Place, Workspace},
    front::{
        App,
        app::Notices,
        keymap::{Action, Keymap, desk},
    },
};

/// Returns the hints that an open menu adds, beside those of the keys of
/// GO and Panic
fn in_menu(app: App) -> Vec<String> {
    hinted(&app)
        .into_iter()
        .filter(|(action, _)| matches!(action, Some(Action::Pick | Action::CloseMenu)))
        .map(|(_, hint)| hint)
        .collect()
}

#[test]
fn an_open_menu_hints_the_keys_that_pick_and_close_it() {
    assert_eq!(
        in_menu(opened(true, false)),
        ["Control+y: pick", "Control+e: close"]
    );
}

#[test]
fn a_closed_menu_and_a_menu_under_the_help_page_hint_nothing() {
    assert!(in_menu(opened(false, false)).is_empty());
    assert!(in_menu(opened(true, true)).is_empty());
}

#[test]
fn hints_of_a_menu_give_way_after_the_rest_when_the_bar_is_short() {
    let hints = owned([
        (Some(Action::Pick), "Control+y: pick"),
        (Some(Action::CloseMenu), "Control+e: close"),
        (Some(Action::Go), "Space: GO"),
        (Some(Action::Help), "F1: help"),
    ]);
    assert_eq!(
        fitted(&hints, 45, &YIELDING),
        "Control+y: pick | Control+e: close | F1: help"
    );
    assert_eq!(
        fitted(&hints, 34, &YIELDING),
        "Control+y: pick | Control+e: close"
    );
    assert_eq!(fitted(&hints, 20, &YIELDING), "Control+y: pick");
}

#[test]
fn the_help_hint_beside_an_open_menu_names_the_first_help_key_of_the_keymap() {
    let help = |keymap| {
        hinted(&opened_with(keymap, true, false))
            .into_iter()
            .find_map(|(action, hint)| (action == Some(Action::Help)).then_some(hint))
    };
    assert_eq!(help(Keymap::default()).as_deref(), Some("?: help"));
    assert_eq!(
        help(desk("help = [\"F1\", \"?\"]")).as_deref(),
        Some("F1: help")
    );
}

#[test]
fn an_open_menu_tells_no_notice_so_that_its_keys_come_first() {
    let app = |menu| App {
        workspace: Workspace::default().apply(Command::Add {
            list: 0,
            kind: Kind::Memo,
            at: Place::End,
            count: 1,
        }),
        notices: Notices {
            remark: Some("The clipboard is empty"),
            ..Notices::default()
        },
        ..opened(menu, false)
    };
    let shown = |menu| fitted(&hinted(&app(menu)), 34, &YIELDING);
    assert_eq!(shown(true), "Control+y: pick | Control+e: close");
    assert!(shown(false).starts_with("The clipboard"));
}
