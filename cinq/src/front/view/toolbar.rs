// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Editing actions above the cue list
//!
//! The list of the kind of cue to add keeps the keys typed while it has
//! the keyboard, as the fields of the inspector do.  A window too narrow for
//! the controls on one line wraps them, in the same order.

use iced::{
    Center, Element, padding,
    widget::{container, row, space, text},
};
use strum::VariantArray;

use super::{SPACING, button, char_width, check_and_leave, dropdown, keep_keys};
use crate::{
    back::{Kind, Place},
    front::{App, Message},
};

/// Columns of the font that the controls take on one line, with a few to
/// spare, below which they wrap
const ONE_LINE: f32 = 84.0;

pub(super) fn view(app: &App, window: f32) -> Element<'_, Message> {
    keep_keys(
        if app.outputs_shown {
            outputs(app, window)
        } else {
            cues(app, window)
        },
        &app.config.keymap,
    )
}

/// Returns the controls of the outputs, where the keys that add and delete a
/// cue add and delete outputs
fn outputs(app: &App, window: f32) -> Element<'_, Message> {
    let several = app.workspace.outputs().len() > 1;
    tools(
        Vec::new(),
        vec![
            show_mode(app),
            button("Add output")
                .on_press_maybe(app.editable().then_some(Message::Add(Place::After, 1)))
                .into(),
            button("Delete output")
                .on_press_maybe(
                    (app.editable() && several && app.output.is_some())
                        .then_some(Message::Delete(1)),
                )
                .into(),
        ],
        window,
    )
}

fn cues(app: &App, window: f32) -> Element<'_, Message> {
    let selected_here = app.selected_here().is_some();
    // Selected cue on standby already leaves nothing to put there
    let movable = selected_here && app.current().standby != app.selected;
    let new_cue = row![
        // Names the list for a newcomer, who took it for the kind of the
        // selected cue, which it only sets for the cue that Add cue makes
        text("New cue:"),
        dropdown(
            Kind::VARIANTS,
            Some(app.new_kind),
            Message::NewKind,
            &app.config.keymap,
        ),
    ]
    .spacing(SPACING)
    .align_y(Center);
    tools(
        vec![
            button("Standby selected")
                .on_press_maybe(movable.then_some(Message::StandbySelected))
                .into(),
        ],
        vec![
            show_mode(app),
            new_cue.into(),
            button("Add cue")
                .on_press_maybe(app.editable().then_some(Message::Add(Place::After, 1)))
                .into(),
            button("Delete cue")
                .on_press_maybe((app.editable() && selected_here).then_some(Message::Delete(1)))
                .into(),
        ],
        window,
    )
}

/// Returns the `left` controls and the `right` ones in a row, with the
/// right ones at the right end where the `window` is wide enough for one line,
/// and otherwise in a row that wraps, which keeps the order
fn tools<'a>(
    left: Vec<Element<'a, Message>>,
    right: Vec<Element<'a, Message>>,
    window: f32,
) -> Element<'a, Message> {
    let one_line = window >= ONE_LINE * char_width();
    let spacer = one_line.then(|| space::horizontal().into());
    let tools = row(left.into_iter().chain(spacer).chain(right))
        .spacing(SPACING)
        .padding(SPACING)
        .align_y(Center);
    if one_line {
        tools.into()
    } else {
        tools.wrap().vertical_spacing(SPACING).into()
    }
}

/// Returns the checkbox of show mode, which locks the buttons beside it,
/// with room after it, so that its label does not read as the start of the
/// next.  Tab reaches it and Enter flips it, and show mode never locks it,
/// since it is the way out.
fn show_mode(app: &App) -> Element<'_, Message> {
    container(check_and_leave(
        "Show mode",
        app.workspace.show_mode(),
        Some(Message::ShowMode),
        &app.config.keymap,
    ))
    .padding(padding::right(SPACING))
    .into()
}
