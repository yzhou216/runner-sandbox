// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the hints of the keys that act where a field, a checkbox, or a
//! drop-down list has the keyboard

mod hinting;
mod notices;

use unicode_width::UnicodeWidthStr;

use super::{
    super::{
        fit::shown,
        hints_in,
        order::{self, YIELDING_IN_FIELD},
    },
    GO, added, before, has_hint, held, hinted, holding, opened, opened_with, texts,
};
use crate::{
    back::{Cue, CueId, Kind, fresh},
    front::{
        App,
        app::{Held, Keyboard},
        keymap::{Action, Keymap, desk, sequence},
    },
};

/// Returns an app in which cue 1, selected, plays `file`, and `keyboard` is
/// where the keyboard is
fn holding_file(keymap: Keymap, file: &str, keyboard: Keyboard) -> App {
    let cue = Cue::new(CueId(1), "1", Kind::Audio, "Rain")
        .file(file)
        .build();
    App {
        keyboard,
        selected: Some(CueId(1)),
        workspace: fresh(vec![cue]),
        ..opened_with(keymap, false, false)
    }
}
