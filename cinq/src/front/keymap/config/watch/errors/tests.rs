// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of writing out the problems of each reading of the file

use indicatif::{InMemoryTerm, MultiProgress, ProgressDrawTarget};

use super::{shown, written};
use crate::front::keymap::config::{Config, Problem};

/// Configuration with a setting that does not exist
fn broken() -> Config {
    Config {
        problems: vec![Problem::new("no setting is named `colour`")],
        ..Config::default()
    }
}

#[test]
fn log_takes_the_problems_of_each_reading_and_nothing_without_them() {
    assert!(written(&broken()).contains("config.toml: no setting is named `colour`"));
    assert_eq!(written(&Config::default()), "");
}

#[test]
fn terminal_keeps_the_latest_problems_below_the_log_until_none_are_left() {
    let screen = InMemoryTerm::new(24, 80);
    let terminal =
        MultiProgress::with_draw_target(ProgressDrawTarget::term_like(Box::new(screen.clone())));
    let bar = shown(&terminal, None, &broken());
    terminal.println("a line of the log").unwrap();
    let contents = screen.contents();
    let log = contents.find("a line of the log");
    let problem = contents.find("no setting is named `colour`");
    assert!(
        matches!((log, problem), (Some(log), Some(problem)) if log < problem),
        "{contents}"
    );
    let bar = shown(&terminal, bar, &Config::default());
    let contents = screen.contents();
    assert!(bar.is_none());
    assert!(
        contents.contains("a line of the log") && !contents.contains("colour"),
        "{contents}"
    );
}
