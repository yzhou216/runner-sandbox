// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the notices that tell a cue to play nowhere and outlast a key

use super::*;
use crate::front::app::{Held, Keyboard};

#[test]
fn the_notice_of_a_cue_gone_silent_outlasts_what_enter_does_but_a_plain_one_does_not() {
    let app = |sends: Option<Sends>| App {
        keyboard: Keyboard::default().with_held(Held {
            any: true,
            check: true,
            ..Held::default()
        }),
        workspace: rain().apply(Command::Edit(CueId(1), Edit::Sends(sends))),
        ..opened(false, false)
    };
    let room = 52;
    let silent = hints_in(&app(Some(Sends::new())), room);
    assert!(silent.contains("play nowhere"), "{silent}");
    assert!(
        silent.contains("leave") && !silent.contains("Enter"),
        "{silent}"
    );
    let plain = hints_in(&app(None), room);
    assert!(plain.contains("Enter: flip"), "{plain}");
}

#[test]
fn only_a_notice_of_silence_that_an_undo_has_not_taken_back_silences() {
    assert!(format::silences("changed cue 2 to play nowhere"));
    assert!(format::silences("1 cue now sends to no output"));
    assert!(!format::silences("undid: changed cue 2 to play nowhere"));
    assert!(!format::silences("changed where cue 2 plays"));
}
