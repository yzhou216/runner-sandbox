// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the configuration file, which chooses a keymap, replaces the
//! keys of the actions it names, sets how far ahead sounds are made ready,
//! and reports what it cannot use

use std::{fs, io, path::Path};

use tempfile::TempDir;

use super::*;
use crate::back::READY_AHEAD;

fn messages(problems: &[config::Problem]) -> Vec<&str> {
    problems
        .iter()
        .map(|problem| problem.message.as_str())
        .collect()
}

#[test]
fn file_chooses_a_keymap_and_replaces_the_keys_of_the_actions_it_names() {
    let (keymap, _, problems) = config::parse(
        "keymap = \"cua\"\n[keys]\ngo = \"F5\"\nnext = [\"n\", \"ArrowDown\", \"n\"]\ndelete = []\n",
    );
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(keymap.name(), "cua");
    assert_eq!(keymap.keys(Action::Go), [keys("F5")]);
    // Keys listed twice for one action are bound once
    assert_eq!(keymap.keys(Action::Next), [keys("n"), keys("ArrowDown")]);
    assert!(keymap.keys(Action::Delete).is_empty());
    assert_eq!(keymap.keys(Action::Undo), [keys("Control+z")]);
}

#[test]
fn file_reports_what_it_cannot_use_and_keeps_the_rest() {
    let (keymap, _, problems) =
        config::parse("colour = 1\n[keys]\nfly = \"f\"\nnext = [\"Foo\", \"n\"]\nlast = \"1 G\"\n");
    assert_eq!(
        messages(&problems),
        [
            "no setting is named `colour`",
            "no action is named `fly`",
            "last: `1 G` starts with a digit, which counts",
            "next: `Foo` names no key",
        ]
    );
    assert_eq!(keymap.keys(Action::Next), [keys("n")]);
    assert!(keymap.keys(Action::Last).is_empty());
}

#[test]
fn file_that_is_not_toml_leaves_the_defaults() {
    let (keymap, _, problems) = config::parse("[keys\n");
    assert_eq!(problems.len(), 1);
    assert_eq!(keymap.keys(Action::Go), Keymap::default().keys(Action::Go));
}

#[test]
fn file_reports_bindings_that_hide_others() {
    let (_, _, problems) = config::parse("[keys]\nhelp = \"g\"\n");
    assert_eq!(
        messages(&problems),
        [
            "`g` comes first, so `g g` can never be typed",
            "`g` comes first, so `g t` can never be typed",
            "`g` comes first, so `g T` can never be typed",
        ]
    );
}

#[test]
fn file_is_in_the_directory_named_on_the_command_line_or_else_that_of_the_platform() {
    assert_eq!(
        config::file(None),
        dirs::config_dir().map(|directory| directory.join("cinq").join("config.toml"))
    );
    // Windows takes a drive for the root of an absolute path
    let show = Path::new(if cfg!(windows) { r"C:\" } else { "/" }).join("show");
    assert_eq!(config::file(Some(&show)), Some(show.join("config.toml")));
    assert!(config::file(Some(Path::new("show"))).is_some_and(|file| file.is_absolute()));
}

#[test]
fn file_defines_keymaps_of_its_own_from_others_or_from_no_keys() {
    let (keymap, _, problems) = config::parse(
        "keymap = \"booth\"\n[keymaps.booth]\nfrom = \"cua\"\ngo = \"F5\"\n[keys]\nhelp = \"F12\"\n",
    );
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(keymap.name(), "booth");
    assert_eq!(keymap.keys(Action::Go), [keys("F5")]);
    assert_eq!(keymap.keys(Action::Undo), [keys("Control+z")]);
    assert_eq!(keymap.keys(Action::Help), [keys("F12")]);
    let (keymap, _, problems) =
        config::parse("keymap = \"mine\"\n[keymaps.mine]\ngo = \"Space\"\npanic = \"Escape\"\n");
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(keymap.keys(Action::Go), [keys("Space")]);
    assert!(keymap.keys(Action::Next).is_empty());
}

#[test]
fn file_reports_what_is_wrong_with_its_keymaps() {
    // Keymap that is not there leaves vi
    let (keymap, _, problems) = config::parse("keymap = \"emacs\"\n");
    assert_eq!(messages(&problems), ["no keymap is named `emacs`"]);
    assert_eq!(keymap.name(), "vi");
    let (keymap, _, problems) =
        config::parse("keymap = \"a\"\n[keymaps.a]\nfrom = \"b\"\n[keymaps.b]\nfrom = \"a\"\n");
    assert_eq!(messages(&problems), ["keymap a starts from itself"]);
    assert_eq!(keymap.name(), "a");
    assert!(keymap.keys(Action::Go).is_empty());
    let (_, _, problems) = config::parse("[keymaps.vi]\ngo = \"F5\"\n");
    assert_eq!(
        messages(&problems),
        ["keymaps.vi goes unused, since vi is a shipped keymap"]
    );
    let (_, _, problems) =
        config::parse("keymap = \"booth\"\n[keymaps.booth]\nfrom = \"emacs\"\nfly = \"f\"\n");
    assert_eq!(
        messages(&problems),
        [
            "no keymap is named `emacs`",
            "keymaps.booth: no action is named `fly`",
        ]
    );
}

#[test]
fn file_sets_the_seconds_ahead_to_make_sounds_ready() {
    let (_, ahead, problems) = config::parse("");
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(ahead, READY_AHEAD);
    let message = "ready_ahead should be a number of seconds, 0 or more";
    let cases = [
        ("10", 10.0, None),
        ("2.5", 2.5, None),
        ("0", 0.0, None),
        ("-1", READY_AHEAD, Some(message)),
        ("\"x\"", READY_AHEAD, Some(message)),
    ];
    cases.into_iter().for_each(|(value, seconds, problem)| {
        let (_, ahead, problems) = config::parse(&format!("ready_ahead = {value}\n"));
        assert_eq!(ahead, seconds, "{value}");
        assert_eq!(messages(&problems), problem.as_slice(), "{value}");
    });
}

#[test]
fn file_made_for_editing_changes_no_key_and_leaves_a_file_there_alone() -> io::Result<()> {
    let home = TempDir::new()?;
    let file = home.path().join("cinq").join("config.toml");
    config::make(&file)?;
    let made = config::load(Some(&file));
    assert!(matches!(&made.source, config::Source::File(read) if *read == file));
    assert!(made.problems.is_empty(), "{:?}", made.problems);
    assert_eq!(made.keymap.name(), Preset::default().name());
    fs::write(&file, "keymap = \"cua\"\n")?;
    config::make(&file)?;
    assert_eq!(fs::read_to_string(&file)?, "keymap = \"cua\"\n");
    Ok(())
}
