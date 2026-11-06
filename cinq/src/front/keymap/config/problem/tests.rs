// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the problems with the configuration file, which point into its
//! text in the sidebar and on standard error

use std::path::PathBuf;

use miette::{GraphicalReportHandler, GraphicalTheme};

use crate::front::keymap::config::{Config, Source, parse};

/// Returns the configuration that a file holding `text` gives
fn read(text: &str) -> Config {
    let (keymap, problems) = parse(text);
    Config {
        keymap,
        source: Source::File(PathBuf::from("/home/operator/.config/cinq/config.toml")),
        text: Some(text.to_owned()),
        problems,
    }
}

/// Returns what each problem says, along with the text that it lies in
fn lying(config: &Config) -> Vec<(&str, Option<&str>)> {
    let text = config.text.as_deref().unwrap_or_default();
    config
        .problems
        .iter()
        .map(|problem| {
            let span = problem.span.clone().map(|span| &text[span]);
            (problem.message.as_str(), span)
        })
        .collect()
}

/// Returns each problem as miette draws it in plain ASCII, so that it does
/// not depend on the terminal that runs the tests
fn drawn(config: &Config) -> Vec<String> {
    let handler = GraphicalReportHandler::new_themed(GraphicalTheme::none());
    config
        .reports()
        .map(|report| {
            let mut drawn = String::new();
            handler
                .render_report(&mut drawn, report.as_ref())
                .expect("a report draws into a string");
            drawn
        })
        .collect()
}

#[test]
fn problems_lie_in_the_part_of_the_file_they_are_about() {
    let config = read(concat!(
        "colour = 1\n",
        "keymap = \"booth\"\n",
        "[keymaps.booth]\n",
        "from = \"emacs\"\n",
        "fly = \"f\"\n",
        "[keymaps.vi]\n",
        "[keys]\n",
        "next = [\"Foo\", 2]\n",
    ));
    assert_eq!(
        lying(&config),
        [
            ("no setting is named `colour`", Some("colour")),
            (
                "keymaps.vi goes unused, since vi is a shipped keymap",
                Some("vi")
            ),
            ("no keymap is named `emacs`", Some("\"emacs\"")),
            ("keymaps.booth: no action is named `fly`", Some("fly")),
            ("next: `Foo` names no key", Some("\"Foo\"")),
            ("next should be keys or a list of keys", Some("2")),
        ]
    );
}

#[test]
fn settings_of_the_wrong_kind_lie_in_their_values() {
    let config = read("keymap = 1\nkeys = \"j\"\nkeymaps = { booth = 2, mine = { from = 3 } }\n");
    assert_eq!(
        lying(&config),
        [
            ("keymap should be the name of a keymap", Some("1")),
            ("keys should be a table", Some("\"j\"")),
        ]
    );
    let config = read("keymap = \"booth\"\nkeymaps = { booth = 2 }\n");
    assert_eq!(
        lying(&config),
        [("keymaps.booth should be a table", Some("2"))]
    );
    let config = read("keymap = \"mine\"\nkeymaps = { mine = { from = 3 } }\n");
    assert_eq!(
        lying(&config),
        [(
            "keymaps.mine.from should be the name of a keymap",
            Some("3")
        )]
    );
}

#[test]
fn keymap_that_starts_from_itself_lies_where_the_loop_closes() {
    let text = "keymap = \"a\"\n[keymaps.a]\nfrom = \"b\"\n[keymaps.b]\nfrom = \"a\"\n";
    let config = read(text);
    assert_eq!(
        lying(&config),
        [("keymap a starts from itself", Some("\"a\""))]
    );
    // Last `"a"`, which the keymap b starts from, rather than the first
    let closing = text.rfind("\"a\"").map(|start| start..start + 3);
    assert_eq!(config.problems[0].span, closing);
}

#[test]
fn file_that_is_not_toml_lies_where_reading_it_stopped() {
    let config = read("keymap = \"cua\"\n[keys\ngo = \"F5\"\n");
    assert_eq!(
        config.warnings().collect::<Vec<_>>(),
        ["config.toml:2: is not TOML: unclosed table, expected `]`"]
    );
}

#[test]
fn sidebar_lists_each_problem_after_the_line_it_lies_on() {
    let config = read("[keys]\nfly = \"f\"\nhelp = \"g\"\n");
    assert_eq!(
        config.warnings().collect::<Vec<_>>(),
        [
            "config.toml:2: no action is named `fly`",
            // Binding that hides another lies in no one place
            "config.toml: `g` comes first, so `g g` can never be typed",
            "config.toml: `g` comes first, so `g t` can never be typed",
            "config.toml: `g` comes first, so `g T` can never be typed",
        ]
    );
}

#[test]
fn standard_error_shows_each_problem_in_the_lines_around_it() {
    let config = read("keymap = \"vi\"\n[keys]\nfly = \"f\"\nhelp = \"g\"\n");
    let drawn = drawn(&config);
    assert!(
        drawn[0].starts_with("  ! config.toml: no action is named `fly`\n"),
        "{}",
        drawn[0]
    );
    assert!(
        drawn[0].contains("[/home/operator/.config/cinq/config.toml:3:1]"),
        "{}",
        drawn[0]
    );
    assert!(
        drawn[0].contains(" 3 | fly = \"f\"\n   : ^^^\n"),
        "{}",
        drawn[0]
    );
    // Problem that lies in no one place shows no lines of the file
    assert_eq!(
        drawn[1],
        "  ! config.toml: `g` comes first, so `g g` can never be typed\n"
    );
}

#[test]
fn standard_error_shows_where_reading_stopped_at_the_end_of_the_file() {
    let drawn = drawn(&read("[keys"));
    assert!(
        drawn[0].starts_with("  ! config.toml: is not TOML: unclosed table, expected `]`\n"),
        "{}",
        drawn[0]
    );
    // Mark past the last character, which the file ends without a line
    // break after
    assert!(
        drawn[0].contains(" 1 | [keys\n   :      ^\n"),
        "{}",
        drawn[0]
    );
}
