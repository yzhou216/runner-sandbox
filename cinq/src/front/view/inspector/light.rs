// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Light patch of the show, and the levels that a light cue fades its
//! instruments to, each listed by instrument

use iced::{
    Center, Element, Fill,
    widget::{column, row, scrollable, space, table, text, text_input},
};

use super::{SPACING, entry, lock, room};
use crate::{
    back::{Cue, Instrument, PatchEdit},
    front::{App, Entry, Message, view::button},
};

/// Characters that the address and level fields hold, as many as `32767/512`
/// or `unchanged` takes, with one to spare
const ROOM: f32 = 10.0;

pub(super) fn patch(app: &App) -> Element<'_, Message> {
    let editable = app.editable();
    let edit = move |edit| editable.then_some(Message::Patch(edit));
    let instruments = table::table(
        [
            table::column(text("Name"), move |instrument: &Instrument| {
                let id = instrument.id;
                text_input("", &instrument.name).on_input_maybe(
                    editable.then_some(move |name| Message::Patch(PatchEdit::Name(id, name))),
                )
            })
            .width(Fill),
            table::column(text("Address"), |instrument: &Instrument| {
                entry(
                    app,
                    Entry::Address(instrument.id),
                    instrument.address_text(),
                    "",
                )
                .width(room(ROOM))
            }),
            table::column(text("Now"), |instrument: &Instrument| {
                text(format!("{:.0}%", app.workspace.light_level(instrument.id)))
            }),
            table::column(text(""), move |instrument: &Instrument| {
                button("Remove").on_press_maybe(edit(PatchEdit::Remove(instrument.id)))
            }),
        ],
        app.workspace.patch(),
    );
    column![
        row![
            text(format!("Light patch of the show{}", lock(app))),
            space::horizontal(),
            button("Add instrument").on_press_maybe(edit(PatchEdit::Add)),
        ]
        .align_y(Center),
        scrollable(instruments).height(Fill),
    ]
    .spacing(SPACING)
    .width(Fill)
    .into()
}

/// Lists every instrument of the patch with its level in the cue, which is
/// empty for those that the cue leaves as they are
pub(super) fn levels<'a>(app: &'a App, cue: &'a Cue) -> Element<'a, Message> {
    if app.workspace.patch().is_empty() {
        return text("The light patch has no instruments yet").into();
    }
    let instruments = table::table(
        [
            table::column(text("Instrument"), |instrument: &Instrument| {
                text(instrument.to_string())
            })
            .width(Fill),
            table::column(text("Level (%)"), move |instrument: &Instrument| {
                let level = cue
                    .lights
                    .get(&instrument.id)
                    .map_or_else(String::new, ToString::to_string);
                entry(app, Entry::Light(cue.id, instrument.id), level, "unchanged")
                    .width(room(ROOM))
            }),
        ],
        app.workspace.patch(),
    );
    scrollable(instruments).height(Fill).into()
}
