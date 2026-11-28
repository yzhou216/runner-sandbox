// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keymaps of a configuration file's own for the tests, which need one that
//! binds no key to cancel, since the vi keymap that cinq ships binds two

use super::Keymap;
use crate::front::config;

/// Returns the keymap that a file declares as `desk` with `body` as its
/// entries, which must raise no problem
pub(in crate::front) fn declared(body: &str) -> Keymap {
    let text = format!("keymap = \"desk\"\n[keymaps.desk]\n{body}\n");
    let (keymap, _, _, problems) = config::parse(&text);
    assert!(problems.is_empty(), "{problems:?}");
    keymap
}

/// Returns the keymap `desk` that starts from vi and binds no key to
/// cancel, with the lines of `bindings` added to its entries
pub(in crate::front) fn desk(bindings: &str) -> Keymap {
    declared(&format!("from = \"vi\"\ncancel = []\n{bindings}"))
}
