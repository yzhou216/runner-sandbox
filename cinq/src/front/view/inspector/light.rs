// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Light patch of the show, and the levels of a light cue

use iced::{
    Center, Element, Fill, Padding,
    widget::{
        button::DEFAULT_PADDING, column, container, responsive, row, scrollable, space, table,
        text, text_input,
    },
};
use unicode_width::UnicodeWidthStr;

use super::{
    SPACING, lock,
    locked::locked,
    output::{help, refusal, styled, typing},
};
use crate::{
    back::{Instrument, PatchEdit, one_line},
    front::{
        App, Message, Setting,
        message::reasons::{address_help, rejection},
        view::{
            ADDRESS, FORMS, button, char_width,
            cue_table::{CELL_SPACE, SCROLL_BAR},
            reached, theme,
            widget::hint,
        },
    },
};

/// Characters that `32768/512` and `unchanged` take, with one to spare
const ROOM: f32 = 10.0;

/// Columns of the font that the name of an instrument keeps before the Now
/// column gives way, enough to read what a name starts with
const NAME_ROOM: f32 = 16.0;

mod levels;
#[cfg(test)]
mod tests;

pub(super) use levels::levels;

pub(super) fn patch(app: &App) -> Element<'_, Message> {
    responsive(move |size| page(app, size.width)).into()
}

/// Returns whether the table of the patch keeps its Now column in `room`
/// pixels, which it does while the Name column keeps [`NAME_ROOM`] columns
/// beside the others
fn now_fits(room: f32) -> bool {
    let button = "Remove".width() as f32 * char_width() + 2.0 * DEFAULT_PADDING.x();
    with_now(room, button + address_room(), 4.0)
}

/// Returns whether a table keeps its Now column in `room` pixels, beside a
/// Name column of [`NAME_ROOM`] columns, `others` pixels of other columns,
/// and the cell spaces of its `columns`
fn with_now(room: f32, others: f32, columns: f32) -> bool {
    let now = "100%".width() as f32 * char_width();
    let name = NAME_ROOM * char_width() + text_input::DEFAULT_PADDING.x();
    name + now + others + columns * CELL_SPACE + SCROLL_BAR <= room
}

fn now_cell<'a>(app: &App, instrument: &Instrument) -> Element<'a, Message> {
    if app
        .workspace
        .patch()
        .iter()
        .all(|patched| patched.id != instrument.id)
    {
        return text("").into();
    }
    text(format!("{:.0}%", app.workspace.light_level(instrument.id))).into()
}

fn address_room() -> f32 {
    super::output::room(ROOM)
}

/// Column of a patch table, of the instrument at each row
type Column<'a> = table::Column<'a, 'a, &'a Instrument, Message>;

fn page(app: &App, width: f32) -> Element<'_, Message> {
    let reason = refusal(app, |setting| matches!(setting, Setting::Address(_)));
    // Below the table, since the column of addresses is too narrow for it.  A
    // row of its own takes the room of an instrument at 1024x768, so it shows
    // only while it helps.
    let help = shows_help(app, &reason).then(|| help(address_help(), reason));
    column![header(app), body(app, width)]
        .extend(help)
        .spacing(SPACING)
        .width(Fill)
        .into()
}

fn header(app: &App) -> Element<'_, Message> {
    row![
        text(format!("Light patch of the show{}", lock(app))),
        space::horizontal(),
        button("Add instrument")
            .on_press_maybe(app.editable().then_some(Message::Patch(PatchEdit::Add)))
            .reachable(&app.config.keymap),
    ]
    .spacing(SPACING)
    .align_y(Center)
    .into()
}

fn body(app: &App, width: f32) -> Element<'_, Message> {
    let instruments = table::table(
        [name_column(app), address_column(app)]
            .into_iter()
            // Gives way to the name, which the operator reads and edits
            .chain(now_fits(width).then(|| now_column(app)))
            .chain([remove_column(app)]),
        app.workspace.patch(),
    );
    if app.workspace.patch().is_empty() {
        text("The light patch has no instruments yet").into()
    } else {
        // Left of the names, room for the labels of hint mode, which the
        // Setting column gives in the other tabs, so that they do not meet
        // the labels of the tabs beside them
        container(scrollable(instruments).id(FORMS).height(Fill))
            .padding(Padding::ZERO.left(hint::width(2)))
            .into()
    }
}

fn name_column(app: &App) -> Column<'_> {
    let editable = app.editable();
    table::column(text("Name"), move |instrument: &Instrument| {
        let id = instrument.id;
        if editable {
            reached(
                text_input("", &instrument.name)
                    .style(theme::field(true))
                    .on_input(move |name| Message::Patch(PatchEdit::Name(id, name))),
            )
        } else {
            locked(&one_line(&instrument.name), "", None, Fill)
        }
    })
    .width(Fill)
}

fn address_column(app: &App) -> Column<'_> {
    table::column(text("Address"), |instrument: &Instrument| {
        field(
            app,
            Setting::Address(instrument.id),
            instrument.address_text(),
            "",
        )
    })
}

fn now_column(app: &App) -> Column<'_> {
    table::column(text("Now"), |instrument: &Instrument| {
        now_cell(app, instrument)
    })
    .align_y(Center)
}

fn remove_column(app: &App) -> Column<'_> {
    let editable = app.editable();
    table::column(text(""), move |instrument: &Instrument| {
        button("Remove")
            .on_press_maybe(editable.then_some(Message::Patch(PatchEdit::Remove(instrument.id))))
            .reachable_and_leave(&app.config.keymap)
    })
}

fn shows_help(app: &App, refusal: &Option<String>) -> bool {
    app.keyboard.held.address || refusal.is_some()
}

/// Returns the field of `setting`, red while the text in it is not applied
fn field<'a>(
    app: &App,
    setting: Setting,
    value: String,
    placeholder: &str,
) -> Element<'a, Message> {
    let (input, shown) = typing(app, setting, placeholder, value, ROOM);
    let input = match setting {
        Setting::Address(_) => input.id(ADDRESS),
        _ => input,
    };
    let valid = rejection(app, setting, &shown).is_none();
    styled(app, input, &shown, placeholder, ROOM, valid)
}
