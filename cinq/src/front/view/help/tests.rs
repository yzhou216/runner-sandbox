// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of which actions the help page lists

use super::bound;
use crate::front::keymap::{Action, Keymap, Preset};

#[test]
fn actions_with_no_key_in_the_keymap_in_use_have_no_row() {
    let vi = bound(&Keymap::from(Preset::Vi));
    assert!(!vi.contains(&Action::Delete));
    assert!(vi.contains(&Action::Cut) && vi.contains(&Action::EditStart));
    let cua = bound(&Keymap::from(Preset::Cua));
    assert!(cua.contains(&Action::Delete));
    assert!(!cua.contains(&Action::HalfPageDown) && !cua.contains(&Action::SearchBackward));
}

#[test]
fn rows_keep_the_order_of_the_actions_with_go_and_panic_first() {
    let vi = bound(&Keymap::from(Preset::Vi));
    assert_eq!(vi[..3], [Action::Go, Action::Panic, Action::Standby]);
}
