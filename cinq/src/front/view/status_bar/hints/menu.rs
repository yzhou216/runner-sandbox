// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Hints while a menu is open

use super::keys::{Hint, Keys};
use crate::front::keymap::Action;

pub(super) fn hints(keys: Keys) -> Vec<Hint> {
    // The page covers an open menu, and takes its keys.  Enter picks
    // whichever key is bound to pick, so that it stands in where none is.
    let pick = keys
        .key(Action::Pick, "pick")
        .or_else(|| Some((Some(Action::Pick), "Enter: pick".to_owned())));
    let close = keys.key(Action::CloseMenu, "close");
    [pick, close].into_iter().flatten().collect()
}
