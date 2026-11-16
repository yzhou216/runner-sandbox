// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Editing actions above the cue list
//!
//! The list of the kind of cue to add keeps the keys typed while it has
//! the keyboard, as the fields of the inspector do.

use iced::{
    Center, Element,
    widget::{row, space, text},
};
use strum::VariantArray;

use super::{SPACING, button, dropdown, keep_keys};
use crate::{
    back::{Kind, Place},
    front::{App, Message},
};

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let keymap = &app.config.keymap;
    if app.outputs_shown {
        // Add output takes the place of Add cue, as the keys of adding and
        // deleting a cue add and delete outputs here
        let several = app.workspace.outputs().len() > 1;
        let outputs = row![
            space::horizontal(),
            button("Add output")
                .on_press_maybe(app.editable().then_some(Message::Add(Place::After, 1))),
            button("Delete output").on_press_maybe(
                (app.editable() && several && app.output.is_some()).then_some(Message::Delete(1)),
            ),
        ]
        .spacing(SPACING)
        .padding(SPACING);
        return outputs.into();
    }
    let selected_here = app.selected_here().is_some();
    // Selected cue on standby already leaves nothing to put there
    let movable = selected_here && app.current().standby != app.selected;
    let tools = row![
        button("Standby selected").on_press_maybe(movable.then_some(Message::StandbySelected)),
        space::horizontal(),
        // Names the list for a newcomer, who took it for the kind of the
        // selected cue, which it only sets for the cue that Add cue makes
        text("New cue:"),
        dropdown(Kind::VARIANTS, Some(app.new_kind), Message::NewKind, keymap),
        button("Add cue").on_press_maybe(app.editable().then_some(Message::Add(Place::After, 1))),
        button("Delete cue")
            .on_press_maybe((app.editable() && selected_here).then_some(Message::Delete(1))),
    ]
    .spacing(SPACING)
    .padding(SPACING)
    .align_y(Center);
    keep_keys(tools, keymap)
}
