// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the most cues that a paste adds

use super::*;

#[test]
fn paste_adds_at_most_a_thousand_cues() {
    let app = after(
        show(Some(CueId(4))),
        [Message::Copy(1), Message::Paste(Place::After, usize::MAX)],
    );
    assert_eq!(rows(&app).len(), 12 + 1000);
}

#[test]
fn a_clipboard_of_more_than_a_thousand_cues_pastes_a_thousand() {
    let app = after(
        crowded(1500),
        [Message::Copy(1500), Message::Paste(Place::After, 1)],
    );
    assert_eq!(rows(&app).len(), 1500 + 1000);
    let app = after(app, [Message::Paste(Place::After, 2)]);
    assert_eq!(rows(&app).len(), 1500 + 2000);
}

#[test]
fn cutting_over_a_motion_pastes_at_most_a_thousand_cues_like_any_cut() {
    use crate::front::keymap::Action::{Cut, Last};
    let cut = operators::across(crowded(1500), Cut, Last, None);
    let app = after(cut, [Message::Paste(Place::After, 2)]);
    assert_eq!(rows(&app).len(), 1000);
}
