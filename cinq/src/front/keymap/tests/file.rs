// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the configuration file, which the keymap reads, and of its
//! problems

mod keymaps;
mod keys;
mod settings;

use super::*;

/// Returns the keymap that a file makes and the problems that it raises
fn parsed(text: &str) -> (Keymap, Vec<config::Problem>) {
    let (keymap, _, _, problems) = config::parse(text);
    (keymap, problems)
}

/// Returns the keymap of a file that raises no problem
fn clean(text: &str) -> Keymap {
    let (keymap, problems) = parsed(text);
    assert!(problems.is_empty(), "{problems:?}");
    keymap
}

fn messages(problems: &[config::Problem]) -> Vec<&str> {
    problems
        .iter()
        .map(|problem| problem.message.as_str())
        .collect()
}
