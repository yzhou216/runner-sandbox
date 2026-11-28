// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the keyboard is, and where the keys of a way back or forward lead,
//! which decide what the hints of the status bar say

use iced::keyboard::{Key, Modifiers, key::Named};

use super::{
    super::format,
    keys::{Hint, Keys},
    menu_open,
};
use crate::{
    back::{Change, Turn, one_line},
    front::{
        App,
        keymap::{Action, Press},
        message::Spot,
        view::widget::shortened,
    },
};

/// Returns whether a text field, a checkbox, or a closed drop-down list has
/// the keyboard, from which a pick or a close by key, and a Tab, leave it, which
/// the stage window never does, as it holds no field
pub(in crate::front::view::status_bar) fn in_field(app: &App) -> bool {
    app.keyboard.held.any
        && !app.help
        && !menu_open(app)
        && app.search.is_none()
        && !app.stage.has_keyboard()
}

/// Returns whether standby lies past the end of the cue list on show while a
/// later list exists, where GO fires nothing
pub(in crate::front::view::status_bar) fn list_ended(app: &App) -> bool {
    app.current().standby.is_none() && app.list + 1 < app.workspace.lists().len()
}

/// Fewest columns of a place that stay, as the name of a cue list may be
/// long, so that the hint of going there still fits in a bar
const PLACE: usize = 16;

/// Names where a way back or forward goes: the outputs, the cue list, or
/// for a cue of the list on show, the cue
fn place(app: &App, spot: Spot) -> String {
    let list = |index: usize| {
        app.workspace
            .lists()
            .get(index)
            .map_or_else(String::new, |list| shortened(&one_line(&list.name), PLACE))
    };
    match spot {
        Spot::Outputs | Spot::Output(_) => "the outputs".to_owned(),
        Spot::List(index) => list(index),
        Spot::Cue(id) => match app.workspace.list_of(id) {
            Some(index) if index == app.list && !app.outputs_shown => {
                app.workspace.cue(id).map_or_else(String::new, |cue| {
                    format!("cue {}", shortened(&format::number(cue), PLACE))
                })
            }
            Some(index) => list(index),
            None => String::new(),
        },
    }
}

pub(super) fn going(app: &App, action: Action, spot: Option<Spot>) -> Option<Hint> {
    let label = format!("{} to {}", action.name(), place(app, spot?));
    Keys(app).key(action, &label)
}

/// Returns whether the empty File field has the keyboard, where Enter opens
/// the chooser of a file and does not leave
pub(in crate::front::view::status_bar) fn choosing(app: &App) -> bool {
    in_field(app)
        && app.keyboard.held.file
        && app
            .selected
            .and_then(|id| app.workspace.cue(id))
            .is_some_and(|cue| cue.file.is_empty())
}

/// Returns whether Enter leaves a text field, which it does unless the keymap
/// binds it to an action that acts in fields, such as GO, which then takes it
pub(super) fn enter_leaves(app: &App) -> bool {
    Press::from_key(&Key::Named(Named::Enter), Modifiers::empty())
        .is_none_or(|enter| !app.config.keymap.acts_in_fields(enter))
}

/// Returns whether a key of `action` acts where the keyboard is.  A text
/// field keeps every key but those of [`Action::acts_in_fields`], and a
/// closed drop-down list those but for [`Action::acts_in_lists`].
pub(in crate::front::view::status_bar) fn acts_here(app: &App, action: Action) -> bool {
    // The stage window holds no field, and leaves the keys of
    // [`Action::stays_in_main_window`] to the main window, as the update does
    if app.stage.has_keyboard() {
        return !action.stays_in_main_window();
    }
    match (app.keyboard.held.text, app.keyboard.held.any) {
        (true, _) => action.acts_in_fields(),
        (false, true) => action.acts_in_lists(),
        (false, false) => true,
    }
}

/// Returns whether the latest change added a cue, whose default name the key
/// of renaming selects whole, while the cue list is in view and no menu takes
/// the keys
pub(super) fn renaming(app: &App) -> bool {
    app.editable()
        && !app.outputs_shown
        && !menu_open(app)
        && matches!(
            app.workspace.latest_change(),
            Some((Turn::Done, Change::Added { .. }))
        )
        && app.workspace.undoable().is_some()
}
