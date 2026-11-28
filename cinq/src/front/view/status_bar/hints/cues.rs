// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Hints of the keys that act on the cue list on show

use super::{
    keys::{Hint, Keys},
    menu_open,
};
use crate::front::{App, keymap::Action};

/// Returns whether the cue list is in view with no field, search, page, or
/// menu holding the keys
fn in_cues(app: &App) -> bool {
    !app.outputs_shown
        && !app.keyboard.held.any
        && !app.help
        && !menu_open(app)
        && app.search.is_none()
}

/// Hints the key that opens a show, as the first step with no show open,
/// which no button of the toolbar teaches by key
pub(super) fn open(keys: Keys) -> Option<Hint> {
    let app = keys.0;
    keys.key(Action::Open, "open")
        .filter(|_| in_cues(app) && !app.show.has_file() && app.workspace.cues().next().is_none())
}

/// Hints the key that moves standby to the selected cue, which no button of
/// the toolbar teaches by key, while the selection is elsewhere
pub(super) fn standby(keys: Keys) -> Option<Hint> {
    let app = keys.0;
    keys.key(Action::Standby, "standby").filter(|_| {
        in_cues(app)
            && app
                .selected_here()
                .is_some_and(|cue| app.current().standby != Some(cue))
    })
}

pub(super) fn tabs(keys: Keys) -> Vec<Hint> {
    let in_cues = in_cues(keys.0);
    // Tabs of the inspector show with the cues, and no field keeps the key
    let tab = keys.key(Action::NextTab, "next tab").filter(|_| in_cues);
    // The way to the next list, and at last to the outputs, which only the
    // keys teach
    let next_list = keys.key(Action::NextList, "next list").filter(|_| in_cues);
    [tab, next_list].into_iter().flatten().collect()
}
