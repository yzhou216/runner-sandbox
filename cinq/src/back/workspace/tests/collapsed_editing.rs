// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of adding, pasting, moving, and indenting cues next to collapsed
//! groups, which count as one row, as the cue list shows them

mod adding;
mod moving;

use super::*;
use crate::back::Change;

/// Returns a collapsed group of two levels, with a cue above and below it
fn closed() -> Workspace {
    nested().apply(collapse(&[2], true))
}

/// Asserts that undo and redo bring back the lists of `before`
/// and `after`, with their depths
fn assert_undoes(before: &Workspace, after: &Workspace) {
    let undone = after.clone().apply(Command::Undo);
    assert_eq!(ids(&undone), ids(before));
    assert_eq!(raw_depths(&undone), raw_depths(before));
    let redone = undone.apply(Command::Redo);
    assert_eq!(ids(&redone), ids(after));
    assert_eq!(raw_depths(&redone), raw_depths(after));
}
