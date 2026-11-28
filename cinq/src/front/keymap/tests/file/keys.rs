// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that the configuration file binds and reports on

use super::*;

#[test]
fn file_chooses_a_keymap_and_replaces_the_keys_of_the_actions_it_names() {
    let keymap = clean(
        "keymap = \"vi\"\n[keys]\ngo = \"F5\"\nnext = [\"m\", \"ArrowDown\", \"m\"]\ncut = []\n",
    );
    assert_eq!(keymap.name(), "vi");
    assert_eq!(keymap.keys(Action::Go), [keys("F5")]);
    // Keys listed twice for one action are bound once
    assert_eq!(keymap.keys(Action::Next), [keys("m"), keys("ArrowDown")]);
    assert!(keymap.keys(Action::Cut).is_empty());
    assert_eq!(keymap.keys(Action::Undo), [keys("u")]);
}

#[test]
fn file_reports_what_it_cannot_use_and_keeps_the_rest() {
    let (keymap, problems) =
        parsed("colour = 1\n[keys]\nfly = \"f\"\nnext = [\"Foo\", \"m\"]\nlast = \"1 G\"\n");
    assert_eq!(
        messages(&problems),
        [
            "no setting is named `colour`",
            "no action is named `fly`",
            "next: `Foo` names no key",
            "last: `1 G` starts with a digit, which counts",
            "last keeps G, Control+End",
        ]
    );
    assert_eq!(keymap.keys(Action::Next), [keys("m")]);
    assert_eq!(
        keymap.keys(Action::Last),
        Keymap::default().keys(Action::Last)
    );
}

#[test]
fn setting_below_a_table_is_told_to_belong_above_it() {
    let (_, _, address, problems) =
        config::parse("[keys]\ngo = \"F5\"\nosc_listen = \"127.0.0.1:53000\"\n");
    assert_eq!(
        messages(&problems),
        ["`osc_listen` is a setting, which belongs above the first table"]
    );
    assert_eq!(
        address, None,
        "the setting is not read from inside the table"
    );
}

#[test]
fn entry_that_no_key_can_be_read_from_keeps_the_keys_of_the_action() {
    let (keymap, problems) =
        parsed("[keys]\ngo = \"Spacebar\"\npanic = [\"Esc\", 1]\ndelete = []\n");
    assert_eq!(
        messages(&problems),
        [
            "go: `Spacebar` names no key",
            "go keeps Space",
            "panic: `Esc` names no key",
            "panic should be keys or a list of keys",
            "panic keeps Escape",
        ]
    );
    assert_eq!(keymap.keys(Action::Go), Keymap::default().keys(Action::Go));
    assert_eq!(
        keymap.keys(Action::Panic),
        Keymap::default().keys(Action::Panic)
    );
    // An empty list is written on purpose
    assert!(keymap.keys(Action::Delete).is_empty());
}

#[test]
fn entry_that_some_key_can_be_read_from_binds_those() {
    let (keymap, problems) = parsed("[keys]\ngo = [\"Spacebar\", \"F5\"]\n");
    assert_eq!(messages(&problems), ["go: `Spacebar` names no key"]);
    assert_eq!(keymap.keys(Action::Go), [keys("F5")]);
}

#[test]
fn file_that_is_not_toml_leaves_the_defaults() {
    let (keymap, problems) = parsed("[keys\n");
    assert_eq!(problems.len(), 1);
    assert_eq!(keymap.keys(Action::Go), Keymap::default().keys(Action::Go));
}

#[test]
fn file_reports_bindings_that_hide_others() {
    let (_, problems) = parsed("[keys]\nhelp = \"g\"\n");
    assert_eq!(
        messages(&problems),
        [
            "`g` comes first, so `g j` can never be typed",
            "`g` comes first, so `g k` can never be typed",
            "`g` comes first, so `g g` can never be typed",
            "`g` comes first, so `g t` can never be typed",
            "`g` comes first, so `g T` can never be typed",
        ]
    );
}
