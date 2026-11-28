// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keymaps that the configuration file chooses and defines

use super::*;

#[test]
fn file_defines_keymaps_of_its_own_from_others_or_from_no_keys() {
    let keymap = clean(
        "keymap = \"booth\"\n[keymaps.booth]\nfrom = \"vi\"\ngo = \"F5\"\n[keys]\nhelp = \"F12\"\n",
    );
    assert_eq!(keymap.name(), "booth");
    assert_eq!(keymap.keys(Action::Go), [keys("F5")]);
    assert_eq!(keymap.keys(Action::Undo), [keys("u")]);
    assert_eq!(keymap.keys(Action::Help), [keys("F12")]);
    let keymap = clean("keymap = \"mine\"\n[keymaps.mine]\ngo = \"Space\"\npanic = \"Escape\"\n");
    assert_eq!(keymap.keys(Action::Go), [keys("Space")]);
    assert!(keymap.keys(Action::Next).is_empty());
}

const RETIRED: &str = "cinq no longer ships the cua keymap, so vi stands in for it";

/// Returns the part of `text` that the first problem lies in
fn lying<'a>(problems: &[config::Problem], text: &'a str) -> Option<&'a str> {
    problems
        .first()
        .and_then(|problem| problem.span.clone())
        .map(|span| &text[span])
}

#[test]
fn file_that_chooses_the_cua_keymap_is_told_so_and_gets_the_keys_of_vi() {
    let text = "keymap = \"cua\"\n";
    let (keymap, problems) = parsed(text);
    assert_eq!(messages(&problems), [RETIRED]);
    assert_eq!(lying(&problems, text), Some("\"cua\""));
    assert_eq!(keymap.name(), "vi");
    [Action::Go, Action::Panic, Action::Next, Action::Cancel]
        .into_iter()
        .for_each(|action| {
            assert_eq!(keymap.keys(action), Keymap::default().keys(action));
        });
}

#[test]
fn keymap_of_the_file_that_starts_from_cua_starts_from_vi_and_the_file_is_told_so() {
    let text = "keymap = \"booth\"\n[keymaps.booth]\nfrom = \"cua\"\ngo = \"F5\"\n";
    let (keymap, problems) = parsed(text);
    assert_eq!(messages(&problems), [RETIRED]);
    assert_eq!(lying(&problems, text), Some("\"cua\""));
    assert_eq!(keymap.name(), "booth");
    assert_eq!(keymap.keys(Action::Go), [keys("F5")]);
    assert_eq!(keymap.keys(Action::Panic), [keys("Escape")]);
    assert_eq!(
        keymap.keys(Action::Next),
        Keymap::default().keys(Action::Next)
    );
}

#[test]
fn file_that_declares_a_keymap_named_cua_uses_it_and_raises_nothing() {
    let keymap =
        clean("keymap = \"cua\"\n[keymaps.cua]\ngo = \"F5\"\n[keymaps.booth]\nfrom = \"cua\"\n");
    assert_eq!(keymap.name(), "cua");
    assert_eq!(keymap.keys(Action::Go), [keys("F5")]);
    assert!(keymap.keys(Action::Next).is_empty());
    let keymap =
        clean("keymap = \"booth\"\n[keymaps.cua]\ngo = \"F5\"\n[keymaps.booth]\nfrom = \"cua\"\n");
    assert_eq!(keymap.keys(Action::Go), [keys("F5")]);
}

#[test]
fn a_line_of_thousands_of_keymaps_each_starting_from_the_next_keeps_the_last_keys() {
    let line: String = (0..5000)
        .map(|at| format!("[keymaps.k{at}]\nfrom = \"k{}\"\n", at + 1))
        .collect();
    let keymap = clean(&format!(
        "keymap = \"k0\"\n{line}[keymaps.k5000]\nfrom = \"vi\"\ngo = \"F5\"\n"
    ));
    assert_eq!(keymap.name(), "k0");
    assert_eq!(keymap.keys(Action::Go), [keys("F5")]);
}

#[test]
fn file_reports_what_is_wrong_with_its_keymaps() {
    // Keymap that is not there leaves vi
    let (keymap, problems) = parsed("keymap = \"emacs\"\n");
    assert_eq!(messages(&problems), ["no keymap is named `emacs`"]);
    assert_eq!(keymap.name(), "vi");
    // Name that only looks like cua
    let (keymap, problems) = parsed("keymap = \"cua2\"\n");
    assert_eq!(messages(&problems), ["no keymap is named `cua2`"]);
    assert_eq!(keymap.keys(Action::Go), Keymap::default().keys(Action::Go));
    let (keymap, problems) =
        parsed("keymap = \"a\"\n[keymaps.a]\nfrom = \"b\"\n[keymaps.b]\nfrom = \"a\"\n");
    assert_eq!(messages(&problems), ["keymap a starts from itself"]);
    assert_eq!(keymap.name(), "a");
    assert!(keymap.keys(Action::Go).is_empty());
    let (_, problems) = parsed(
        "keymap = \"a\"\n[keymaps.a]\nfrom = \"b\"\n[keymaps.b]\nfrom = \"c\"\n[keymaps.c]\nfrom = \"b\"\n",
    );
    assert_eq!(messages(&problems), ["keymap b starts from itself"]);
    let (_, problems) = parsed("[keymaps.vi]\ngo = \"F5\"\n");
    assert_eq!(
        messages(&problems),
        ["keymaps.vi goes unused, since vi is a shipped keymap"]
    );
    let (_, problems) =
        parsed("keymap = \"booth\"\n[keymaps.booth]\nfrom = \"emacs\"\nfly = \"f\"\n");
    assert_eq!(
        messages(&problems),
        [
            "no keymap is named `emacs`",
            "keymaps.booth: no action is named `fly`",
        ]
    );
}

#[test]
fn keymap_that_no_setting_uses_is_checked_as_the_one_in_use_is() {
    let text = "[keymaps.booth]\nfrom = \"emacs\"\nfly = \"f\"\n[keymaps.mine]\nfrom = 3\ngo = \"Spacebar\"\n";
    let (keymap, problems) = parsed(text);
    assert_eq!(
        messages(&problems),
        [
            "no keymap is named `emacs`",
            "keymaps.booth: no action is named `fly`",
            "keymaps.mine.from should be the name of a keymap",
            "keymaps.mine: go: `Spacebar` names no key",
        ]
    );
    assert_eq!(keymap.name(), "vi");
    // A problem that the keymap in use shares with another is told once
    let (_, problems) = parsed("keymap = \"booth\"\n[keymaps.booth]\nfly = \"f\"\n");
    assert_eq!(
        messages(&problems),
        ["keymaps.booth: no action is named `fly`"]
    );
}

#[test]
fn value_under_keymaps_that_is_not_a_table_is_told_with_its_line() {
    let text = "[keymaps]\nready_ahead = 5\nbooth = 2\n";
    let (_, problems) = parsed(text);
    assert_eq!(
        messages(&problems),
        [
            "`ready_ahead` is a setting, which belongs above the first table",
            "keymaps.booth should be a table",
        ]
    );
    assert_eq!(
        problems
            .iter()
            .filter_map(|problem| problem.span.clone())
            .map(|span| &text[span])
            .collect::<Vec<_>>(),
        ["ready_ahead", "2"]
    );
}
