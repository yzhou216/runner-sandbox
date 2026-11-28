// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of operators, which take a motion and counts

mod completing;
mod describing;
mod indenting;
mod rebinding;

use super::*;

/// Returns the operator, the motion, and the counts that the keys complete
fn across(text: &str) -> Option<(Action, Action, Option<usize>)> {
    typed(text).across()
}

fn assert_across(wanted: &[(&str, Action, Action, Option<usize>)]) {
    wanted.iter().for_each(|&(text, operator, motion, count)| {
        assert_eq!(across(text), Some((operator, motion, count)), "{text}");
    });
}
