// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of where the problems with the configuration file point in its
//! text

use std::path::PathBuf;

use miette::{GraphicalReportHandler, GraphicalTheme};

use crate::front::config::{Config, Source, parse};

fn read(text: &str) -> Config {
    let (keymap, sound, osc_listen, problems) = parse(text);
    Config {
        keymap,
        sound,
        osc_listen,
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

/// Returns the problems as miette draws them in plain ASCII, so that they
/// do not depend on the terminal that runs the tests
fn draw(config: &Config) -> String {
    let handler = GraphicalReportHandler::new_themed(GraphicalTheme::none());
    config.reports().fold(String::new(), |mut drawn, report| {
        handler
            .render_report(&mut drawn, report.as_ref())
            .expect("a report draws into a string");
        drawn
    })
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
            ("no keymap is named `emacs`", Some("\"emacs\"")),
            ("keymaps.booth: no action is named `fly`", Some("fly")),
            (
                "keymaps.vi goes unused, since vi is a shipped keymap",
                Some("vi")
            ),
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
            ("keymaps.booth should be a table", Some("2")),
            (
                "keymaps.mine.from should be the name of a keymap",
                Some("3")
            ),
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
fn sidebar_lists_each_problem_after_the_line_it_lies_on() {
    let warnings = |text: &str| read(text).warnings().collect::<Vec<_>>();
    assert_eq!(
        warnings("[keys]\nfly = \"f\"\nhelp = \"g\"\n"),
        [
            "config.toml:2: no action is named `fly`",
            // Binding that hides another lies in no one place
            "config.toml: `g` comes first, so `g j` can never be typed",
            "config.toml: `g` comes first, so `g k` can never be typed",
            "config.toml: `g` comes first, so `g g` can never be typed",
            "config.toml: `g` comes first, so `g t` can never be typed",
            "config.toml: `g` comes first, so `g T` can never be typed",
        ]
    );
    assert_eq!(
        warnings("keymap = \"vi\"\n[keys\n"),
        ["config.toml:2: is not TOML: unclosed table, expected `]`"]
    );
    assert_eq!(
        warnings("# Keys\nkeymap = \"cua\"\n"),
        ["config.toml:2: cinq no longer ships the cua keymap, so vi stands in for it"]
    );
}

#[test]
fn standard_error_shows_each_problem_in_the_lines_around_it() {
    let drawn = draw(&read(
        "keymap = \"vi\"\n[keys]\nfly = \"f\"\nhelp = \"g\"\n",
    ));
    let parts = [
        "  ! config.toml: no action is named `fly`\n",
        "[/home/operator/.config/cinq/config.toml:3:1]",
        " 3 | fly = \"f\"\n   : ^^^\n",
        // Problem that lies in no one place shows no lines of the file
        "can never be typed\n  ! config.toml: `g` comes first, so `g t`",
    ];
    assert!(parts.iter().all(|part| drawn.contains(part)), "{drawn}");
    // Mark past the last character of a file without a final line break
    let drawn = draw(&read("[keys"));
    assert!(drawn.contains(" 1 | [keys\n   :      ^\n"), "{drawn}");
}

#[test]
fn a_key_bound_twice_lies_in_the_entry_that_cannot_be_reached() {
    let config = read("keymap = \"vi\"\n[keys]\ngo = [\"F9\"]\nnext = [\"F9\"]\n");
    assert_eq!(
        lying(&config),
        [(
            "`F9` is bound to both go and next, so go keeps it and next loses it",
            Some("next")
        )]
    );
    assert_eq!(
        config.warnings().collect::<Vec<_>>(),
        ["config.toml:4: `F9` is bound to both go and next, so go keeps it and next loses it"]
    );
}

#[test]
fn problems_come_in_the_order_of_the_file() {
    let config = read("keymap = \"vi\"\n[keys]\nfly = \"f\"\ngo = [\"F9\"]\nnext = [\"F9\"]\n");
    let lines = config
        .warnings()
        .map(|warning| warning.split(':').nth(1).unwrap_or_default().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(lines, ["3", "5"]);
}

#[test]
fn a_key_that_cuts_two_default_sequences_short_tells_it_once() {
    // Control+y scrolls the cue list up and picks in a menu
    let config = read("keymap = \"vi\"\n[keys]\ngo = [\"y\"]\n");
    let said = config.warnings().filter(|said| said.contains("Control+y"));
    assert_eq!(
        said.collect::<Vec<_>>(),
        ["config.toml: `y` acts at once as go, so `Control+y` can never be typed"]
    );
}
