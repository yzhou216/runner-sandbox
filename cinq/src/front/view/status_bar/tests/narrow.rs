// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of a bar too narrow for its text, down to none

use unicode_width::UnicodeWidthStr;

use super::{
    super::{fit::shown, name},
    GO, YIELDING, add, before, opened,
};
use crate::{
    back::Workspace,
    front::{App, update::Show},
};

#[test]
fn a_notice_of_wide_and_accented_characters_never_runs_past_any_room() {
    let told = "Changed the name of cue Überraschung, 日本語のキュー | u: undo";
    (0..=told.width() + 12).for_each(|room| {
        let bar = shown(before([GO]), Some(told.to_owned()), room, &YIELDING, &[]);
        assert!(bar.width() <= room, "{bar:?} in {room} columns");
    });
}

#[test]
fn the_hints_beside_a_name_never_run_past_any_room() {
    let app = App {
        show: Show::in_file("日本語のとても長い名前.cinq.toml"),
        workspace: Workspace::default().apply(add()),
        ..opened(false, false)
    };
    (0..120).for_each(|room| {
        let (_, hints) = name::shown(&app, room);
        assert!(hints.width() <= room, "{hints:?} in {room} columns");
    });
}
