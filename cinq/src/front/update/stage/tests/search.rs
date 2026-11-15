// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the search line while the stage window has the keyboard

use super::show;
use crate::{back::Kind, front::Message};

#[test]
fn search_line_takes_no_keyboard_while_the_stage_window_has_it() {
    let (open, _) = show(Kind::Video).staged();
    let stage = open.stage.window().expect("the stage window opens");
    let main = open.main.expect("the main window opens");
    let (staged, _) = open.handled(Message::WindowFocused(stage));
    let (staged, searching) = staged.searching(false);
    assert_eq!((staged.search.is_none(), searching.units()), (true, 0));
    let (back, _) = staged.handled(Message::WindowFocused(main));
    let (back, searching) = back.searching(false);
    assert_eq!((back.search.is_some(), searching.units()), (true, 1));
}
