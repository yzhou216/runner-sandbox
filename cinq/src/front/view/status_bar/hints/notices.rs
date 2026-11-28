// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the status bar tells before its keys: a remark, a field left
//! unapplied, the latest copy or cut, and the latest change to the show

use std::iter;

use itertools::Itertools;

use super::{
    super::{LINE, format},
    acts_here, menu_open,
};
use crate::{
    back::{Change, CueId, Turn},
    front::{App, keymap::Action, view::answered},
};

/// Tells what a key remarked, why a field was left unapplied, or the latest
/// copy or cut, opening or saving, or change to the show told, whichever is
/// latest, and nothing while a menu is open, whose keys of picking and closing
/// come before any notice, since a notice outlasts every key when the bar is
/// short
pub(in crate::front::view::status_bar) fn told(app: &App) -> Option<String> {
    asked(app)
        .or_else(|| notice(app))
        .filter(|_| !menu_open(app))
}

/// Tells what the operator asked for or what went wrong, which comes before
/// the latest change to the show
pub(super) fn asked(app: &App) -> Option<String> {
    clipped(app)
        .or_else(|| app.notices.remark.map(str::to_owned))
        .or_else(|| app.notices.unapplied.clone())
        .or_else(|| app.notices.told.clone())
}

/// Tells what the latest copy or cut took, with the key that pastes it and,
/// for a cut, the key that takes it back
pub(in crate::front::view::status_bar) fn clipped(app: &App) -> Option<String> {
    let keymap = &app.config.keymap;
    let clipped = app.notices.clipped.as_ref()?;
    let paste = keymap
        .hint(Action::PasteBelow, "paste")
        .filter(|_| app.editable() && acts_here(app, Action::PasteBelow));
    let undo = keymap.hint(Action::Undo, Action::Undo.name()).filter(|_| {
        clipped.cut && app.workspace.undoable().is_some() && acts_here(app, Action::Undo)
    });
    Some(
        iter::once(clipped.told.clone())
            .chain(paste)
            .chain(undo)
            .join(LINE),
    )
}

/// Tells the latest change to the show, with the key that takes it back,
/// or that makes it again after an undo, while that key can act.  While the
/// outputs show, only a change to the outputs is told, since a notice about
/// cues that are out of sight would take the room of the keys of the outputs,
/// which notices outrank when the bar is short.
pub(in crate::front::view::status_bar) fn notice(app: &App) -> Option<String> {
    let workspace = &app.workspace;
    let (turn, change) = workspace
        .latest_change()
        .filter(|(_, change)| !app.outputs_shown || matches!(change, Change::Outputs { .. }))?;
    let told = format::change(workspace, change);
    let told = match (turn, change) {
        (Turn::Done, Change::Edited { cue, .. }) => stopping(app, told, &[*cue]),
        (Turn::Done, Change::Deleted { cues, .. }) => stopping(
            app,
            told,
            &cues.iter().map(|placed| placed.cue.id).collect_vec(),
        ),
        _ => told,
    };
    let facts = format::consequences_of(workspace, change);
    let (told, action, next) = match turn {
        Turn::Done => (told, Action::Undo, workspace.undoable()),
        Turn::Undone => (
            format::turned("undid", app.notices.turns, &told),
            Action::Redo,
            workspace.redoable(),
        ),
        Turn::Redone => (
            format::turned("redid", app.notices.turns, &told),
            Action::Undo,
            workspace.undoable(),
        ),
    };
    let told = format::leading(facts, &told);
    // A text field keeps every key, so that no key takes the change back
    // there, while a closed list lets the keys of undo and redo act
    next.map(|_| {
        if acts_here(app, action) {
            answered(told, &app.config.keymap, action)
        } else {
            told
        }
    })
}

/// Tells after `told`, the notice of a change to `cues`, which of them the
/// change stopped
fn stopping(app: &App, told: String, cues: &[CueId]) -> String {
    let stopped = app
        .notices
        .stopped
        .iter()
        .filter(|cue| cues.contains(cue))
        .count();
    format::stopping(told, cues.len(), stopped)
}
