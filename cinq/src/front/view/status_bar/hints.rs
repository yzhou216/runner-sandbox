// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Hints of the status bar in the order that the bar shows them

mod cues;
mod fields;
mod general;
mod hint_mode;
pub(super) mod keyboard;
mod keys;
mod menu;
pub(super) mod notices;

use keyboard::going;
pub(super) use keyboard::{acts_here, choosing, in_field, list_ended};
use keys::Keys;

use crate::front::{
    App,
    keymap::{Action, shown},
};

#[cfg(test)]
pub(super) fn hints(app: &App) -> Vec<(Option<Action>, String)> {
    hints_told(app, notices::told(app))
}

/// Returns whether a menu is open, which the help page covers
fn menu_open(app: &App) -> bool {
    app.keyboard.menu && !app.help
}

/// Returns the hints, each with the action whose keys it names when it can
/// give way, with `told` for what the latest change told
pub(super) fn hints_told(app: &App, told: Option<String>) -> Vec<(Option<Action>, String)> {
    let keys = Keys(app);
    if app.hints.is_some() {
        return hint_mode::hints(app, keys);
    }
    let typed = (!app.keys.is_empty()).then(|| (None, shown(&app.keys)));
    let told = told.map(|told| (None, told));
    let back = going(app, Action::Back, app.back_to());
    let forward = going(app, Action::Forward, app.forward_to());
    let save = keys
        .key(Action::Save, Action::Save.name())
        .filter(|_| app.show.unsaved());
    typed
        .into_iter()
        .chain(told)
        .chain(
            menu_open(app)
                .then(|| menu::hints(keys))
                .into_iter()
                .flatten(),
        )
        .chain(fields::hints(keys))
        .chain(general::teaching(keys))
        .chain(general::rename(keys))
        .chain(back)
        .chain(forward)
        .chain(save)
        .chain(cues::open(keys))
        .chain(general::go(keys))
        .chain(keys.key(Action::Panic, "Panic"))
        .chain(general::stage(keys))
        .chain(general::help(keys))
        .chain(cues::standby(keys))
        .chain(cues::tabs(keys))
        .chain(general::hint_mode(keys))
        .collect()
}
