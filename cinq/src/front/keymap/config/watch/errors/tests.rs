// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of writing out the problems of each reading of the file

use super::{CLEAR, ENTER, LEAVE, left, switched, written};
use crate::front::keymap::config::{Config, Problem};

fn broken() -> Config {
    Config {
        problems: vec![Problem::new("no setting is named `colour`")],
        ..Config::default()
    }
}

#[test]
fn problems_take_a_screen_of_their_own_until_none_are_left() {
    let fine = Config::default();
    let entered = switched(false, &broken());
    assert!(
        entered.starts_with(&format!("{ENTER}{CLEAR}")),
        "{entered:?}"
    );
    assert!(entered.contains("config.toml: no setting is named `colour`"));
    let redrawn = switched(true, &broken());
    assert!(
        redrawn.starts_with(CLEAR) && !redrawn.contains(ENTER),
        "{redrawn:?}"
    );
    assert_eq!(switched(true, &fine), LEAVE);
    assert_eq!(switched(false, &fine), "");
}

#[test]
fn problems_left_at_the_end_follow_the_main_screen_back() {
    let left = left(&broken());
    assert!(left.starts_with(LEAVE), "{left:?}");
    assert!(left.contains("config.toml: no setting is named `colour`"));
}

#[test]
fn log_takes_the_problems_of_each_reading_and_nothing_without_them() {
    assert!(written(&broken()).contains("config.toml: no setting is named `colour`"));
    assert_eq!(written(&Config::default()), "");
}
