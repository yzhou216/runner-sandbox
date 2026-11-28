// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of picking the kind of the cues to add, after which the keyboard goes
//! back to the cue list so that the key of adding acts at once

use crate::{
    back::Kind,
    front::{App, Message},
};

#[test]
fn a_pick_takes_the_keyboard_from_the_list_and_gives_it_to_the_cue_list() {
    let (app, task) = App::default().handled(Message::NewKind(Kind::Fade));
    assert_eq!(app.new_kind, Kind::Fade);
    assert!(task.units() > 0);
}
