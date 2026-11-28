// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Levels of a light cue, a row for each instrument

use iced::{
    Center, Element, Fill,
    widget::{column, responsive, scrollable, table, text},
};

use super::{address_room, field, now_cell, with_now};
use crate::{
    back::{Cue, Instrument},
    front::{
        App, Message, Setting, Tab,
        view::{
            FORMS,
            inspector::{
                SPACING,
                output::{help, refusal},
            },
        },
    },
};

pub(super) fn levels_now_fits(room: f32) -> bool {
    with_now(room, address_room(), 3.0)
}

pub(super) fn no_instruments() -> String {
    format!(
        "The light patch has no instruments yet.  Add one on the {} tab",
        Tab::Patch
    )
}

pub(in super::super) fn levels<'a>(app: &'a App, cue: &'a Cue) -> Element<'a, Message> {
    if app.workspace.patch().is_empty() && cue.lights.is_empty() {
        return text(no_instruments()).into();
    }
    responsive(move |size| levels_page(app, cue, size.width)).into()
}

/// Returns the rows of the levels of `cue`: the instruments of the patch, then
/// each that the cue names and the patch lacks, so that the operator can read
/// and clear a level that no instrument takes
pub(super) fn rows(patch: &[Instrument], cue: &Cue) -> Vec<Instrument> {
    patch
        .iter()
        .cloned()
        .chain(
            cue.lights
                .keys()
                .filter(|id| patch.iter().all(|instrument| instrument.id != **id))
                .map(|id| Instrument {
                    id: *id,
                    name: format!("{} (unpatched)", id.0),
                    universe: 1,
                    address: 1,
                }),
        )
        .collect()
}

fn levels_page<'a>(app: &'a App, cue: &'a Cue, width: f32) -> Element<'a, Message> {
    // Beside the level that the cue sets, the level that the stage has now,
    // which gives way first as in the patch
    let now = table::column(text("Now"), |instrument: Instrument| {
        now_cell(app, &instrument)
    })
    .align_y(Center);
    // Columns take the room of their text, so that a field stays near its
    // name instead of across the width
    let instruments = table::table(
        [
            table::column(text("Instrument"), |instrument: Instrument| {
                text(instrument.to_string())
            })
            .align_y(Center),
            table::column(text("Level (%)"), move |instrument: Instrument| {
                let level = cue
                    .lights
                    .get(&instrument.id)
                    .map_or_else(String::new, ToString::to_string);
                field(
                    app,
                    Setting::Light(cue.id, instrument.id),
                    level,
                    "unchanged",
                )
            }),
        ]
        .into_iter()
        .chain(levels_now_fits(width).then_some(now)),
        rows(app.workspace.patch(), cue),
    );
    // Under the table, since a reason wrapped in a cell would move the rows
    let reason = refusal(
        app,
        |setting| matches!(setting, Setting::Light(id, _) if *id == cue.id),
    );
    column![scrollable(instruments).id(FORMS).width(Fill).height(Fill)]
        .extend(reason.map(|reason| help("", Some(reason))))
        .spacing(SPACING)
        .into()
}
