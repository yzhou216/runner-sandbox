// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the search line while the stage window has the keyboard

use super::{focusing, opened};

#[test]
fn search_line_takes_no_keyboard_while_the_stage_window_has_it() {
    let (staged, searching) = focusing(opened(), true).searching(false);
    assert_eq!((staged.search.is_none(), searching.units()), (true, 0));
    let (back, searching) = focusing(staged, false).searching(false);
    assert_eq!((back.search.is_some(), searching.units()), (true, 1));
}
