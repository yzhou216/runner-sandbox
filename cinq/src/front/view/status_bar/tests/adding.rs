// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the hint that follows a cue just added

use unicode_width::UnicodeWidthStr;

use super::{YIELDING, add, added, fitted, opened, opened_with, owned, texts};
use crate::{
    back::{Command, Workspace},
    front::{
        App,
        app::{Held, Keyboard},
        keymap::{Action, Keymap, desk},
    },
};

/// Returns the first two hints, which tell the latest change and then
/// the key that renames the cue, after `command`
fn told(keymap: Keymap, command: Command, text_held: bool) -> Vec<String> {
    let app = App {
        workspace: Workspace::default().apply(command),
        keyboard: Keyboard::default().with_held(Held {
            text: text_held,
            ..Held::default()
        }),
        ..opened_with(keymap, false, false)
    };
    texts(&app).into_iter().take(2).collect()
}

#[test]
fn a_cue_just_added_hints_the_key_that_renames_it_after_the_key_of_undo() {
    assert_eq!(
        told(Keymap::default(), add(), false),
        ["Added cue 1 | u: undo", "cc: rename"]
    );
    assert_eq!(
        told(desk("change_name = \"F2\""), add(), false),
        ["Added cue 1 | u: undo", "F2: rename"]
    );
}

#[test]
fn a_text_field_that_keeps_every_key_hints_neither_undo_nor_rename() {
    assert_eq!(told(Keymap::default(), add(), true)[0], "Added cue 1");
    assert_eq!(told(desk(""), add(), true)[0], "Added cue 1");
    assert!(
        told(Keymap::default(), add(), true)
            .iter()
            .all(|hint| !hint.contains("rename"))
    );
}

#[test]
fn the_show_mode_that_locks_the_name_hints_no_rename() {
    let locked = added().apply(Command::ShowMode(true));
    let app = App {
        workspace: locked,
        ..opened(false, false)
    };
    assert!(texts(&app).iter().all(|hint| !hint.contains("rename")));
}

#[test]
fn the_outputs_hint_the_rename_of_an_output_and_not_that_of_the_cue_just_added() {
    let app = App {
        workspace: added(),
        outputs_shown: true,
        ..opened(false, false)
    };
    let hints = texts(&app);
    assert!(hints.contains(&"i: rename".to_owned()), "{hints:?}");
    assert!(!hints.contains(&"cc: rename".to_owned()), "{hints:?}");
}

#[test]
fn go_panic_save_and_help_give_way_before_the_rename_of_a_cue_just_added() {
    let hints = owned([
        (None, "Added cue 1 | u: undo"),
        (Some(Action::ChangeName), "cc: rename"),
        (Some(Action::Save), "Control+s: save"),
        (Some(Action::Go), "Space: GO"),
        (Some(Action::Panic), "Escape: Panic"),
        (Some(Action::Help), "?: help"),
    ]);
    let kept = "Added cue 1 | u: undo | cc: rename";
    assert_eq!(fitted(&hints, kept.width(), &YIELDING), kept);
    assert_eq!(
        fitted(&hints, kept.width() - 1, &YIELDING),
        "Added cue 1 | u: undo"
    );
    let with_help = "Added cue 1 | u: undo | cc: rename | ?: help";
    assert_eq!(fitted(&hints, with_help.width(), &YIELDING), with_help);
    let with_save = "Added cue 1 | u: undo | cc: rename | Control+s: save | ?: help";
    assert_eq!(fitted(&hints, with_save.width(), &YIELDING), with_save);
}

#[test]
fn an_open_menu_hints_no_rename_since_its_keys_never_reach_the_name() {
    let app = App {
        workspace: added(),
        ..opened(true, false)
    };
    assert!(texts(&app).iter().all(|hint| !hint.contains("rename")));
}
