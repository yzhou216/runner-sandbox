// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of writing out the problems of each reading of the file

use super::{CLEAR, written};
use crate::front::keymap::config::{Config, Problem};

#[test]
fn screen_shows_the_latest_problems_alone_and_no_problems_write_nothing() {
    let broken = Config {
        problems: vec![Problem::new("no setting is named `colour`")],
        ..Config::default()
    };
    let fine = Config::default();
    // Problems shown first leave what the terminal showed before them
    let first = written(&broken, true, false);
    assert!(first.contains("config.toml: no setting is named `colour`") && !first.contains(CLEAR));
    assert!(written(&broken, true, true).starts_with(CLEAR));
    assert_eq!(written(&fine, true, true), CLEAR);
    assert_eq!(written(&fine, true, false), "");
    assert_eq!(written(&fine, false, true), "");
    assert!(!written(&broken, false, true).contains(CLEAR));
}
