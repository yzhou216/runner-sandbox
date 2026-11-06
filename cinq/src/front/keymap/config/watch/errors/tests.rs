// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of writing out the problems of each reading of the file, which
//! take the place of those before on the screen of a terminal and follow
//! them in a log

use super::{CLEAR, NONE_LEFT, Told, written};
use crate::front::keymap::config::{Config, Problem};

/// Returns a configuration whose file has a problem
fn broken() -> Config {
    Config {
        problems: vec![Problem::new("no setting is named `colour`")],
        ..Config::default()
    }
}

#[test]
fn screen_shows_the_problems_of_the_latest_reading_alone() {
    let (first, told) = written(&broken(), Told::Nothing, true);
    assert!(first.contains("config.toml: no setting is named `colour`"));
    // First problems leave what the terminal showed before them in place
    assert!(!first.starts_with(CLEAR), "{first:?}");
    let (again, told) = written(&broken(), told, true);
    assert!(again.starts_with(CLEAR) && again.contains("`colour`"));
    let (fixed, told) = written(&Config::default(), told, true);
    assert_eq!(fixed, format!("{CLEAR}{NONE_LEFT}"));
    let (still, told) = written(&Config::default(), told, true);
    assert_eq!((still.as_str(), told), ("", Told::NoProblems));
}

#[test]
fn log_takes_the_problems_of_each_reading_after_those_before() {
    let (first, told) = written(&broken(), Told::Nothing, false);
    let (again, told) = written(&broken(), told, false);
    assert!(!first.contains(CLEAR) && !again.contains(CLEAR));
    assert!(again.contains("`colour`"));
    let (fixed, told) = written(&Config::default(), told, false);
    assert_eq!(fixed, NONE_LEFT);
    let (still, _) = written(&Config::default(), told, false);
    assert_eq!(still, "");
}

#[test]
fn file_without_problems_writes_nothing_until_it_has_some() {
    let (text, told) = written(&Config::default(), Told::Nothing, true);
    assert_eq!((text.as_str(), told), ("", Told::Nothing));
}
