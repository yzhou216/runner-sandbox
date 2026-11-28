// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Levels at which an audio or video cue sends to each output of the show
//!
//! Each output has a field that holds the level that the cue sends at, or
//! nothing where the cue does not send there.  Typing a level sends the cue
//! there, and emptying the field takes the send away.  The fields lie in rows
//! that wrap and scroll, so that every output stays in reach, as the 32
//! outputs of a Dante Virtual Soundcard need.  The name of an output that
//! cannot play now shows in yellow where cinq tries its device again by
//! itself, and in red where the output must change, followed in words by
//! what is wrong with it, and the header counts the outputs that the cue
//! sends to and that cannot, so that the color is never the only sign.

#[cfg(test)]
mod tests;

use iced::{
    Center, Element, Fill, Length,
    widget::{column, row, scrollable, text, text::Wrapping},
};

use super::{
    LEVEL_ROOM,
    output::{refusal, refused, styled, typing},
};
use crate::{
    back::{Cue, Fault, OutputId, Sends, one_line},
    front::{
        App, Message, Setting,
        message::reasons::rejection,
        view::{
            SENDS, SPACING, button, char_width, faults, format, widest,
            widget::shortened_start as cut,
        },
    },
};

/// Returns the faults of the outputs that cannot play and that the cue
/// sends to, since an output that it sends nothing to cannot spoil it
fn silent<'a>(app: &'a App, sends: &Sends) -> Vec<&'a Fault> {
    app.workspace
        .outputs()
        .iter()
        .filter(|output| sends.contains_key(&output.id))
        .filter_map(|output| faults::fault(app, output.id))
        .collect()
}

/// Most columns that the name of an output keeps in the sends, which lie in
/// rows that wrap, so that more of them show.  The names take the room of
/// the widest, up to this.  A fault told after the name is not cut, and takes
/// the room that it needs.
const NAME_MOST: usize = 32;

/// Space between the entries of a row, wider than the space inside an entry,
/// where the name is padded to the longest, so that a name reads with its own
/// field and not with the unit before it
const ENTRY_SPACING: f32 = 3.0 * SPACING;
const _: () = assert!(ENTRY_SPACING > 2.0 * SPACING);

/// Returns the name of an output, cut to `columns` columns, with its fault,
/// if it has one, in words, as `House L (not found)`, so that the color of
/// the name is not the only sign
fn named(name: &str, columns: usize, fault: Option<&Fault>) -> String {
    let name = cut(name, columns);
    match fault {
        Some(fault) => format!("{name} ({})", fault.brief()),
        None => name,
    }
}

/// Returns the columns that the names of the outputs of `app` take in the
/// sends, from their plain names, and each name as told, with its fault in
/// words where it has one.  A fault does not widen the room, so that one
/// output that fails leaves the other entries as narrow as they were.
fn labels(app: &App) -> (usize, Vec<(String, Option<&Fault>)>) {
    let outputs = app.workspace.outputs();
    let shown: Vec<String> = outputs
        .iter()
        .map(|output| one_line(&output.name))
        .collect();
    let columns = widest(&shown).min(NAME_MOST);
    let plain: Vec<String> = shown.iter().map(|name| cut(name, columns)).collect();
    let names = outputs
        .iter()
        .zip(&shown)
        .map(|(output, name)| {
            let fault = faults::fault(app, output.id);
            (named(name, columns, fault), fault)
        })
        .collect();
    (widest(&plain), names)
}

pub(super) fn view<'a>(app: &'a App, cue: &'a Cue) -> Element<'a, Message> {
    let sends = app.workspace.sends(cue);
    let (room, names) = labels(app);
    // Names take the room of the longest plain one, so that the fields line up
    // and every send takes the same width in the rows that wrap, while a name
    // that carries fault words takes what it needs
    let name_room = char_width() * room as f32;
    let outputs = app
        .workspace
        .outputs()
        .iter()
        .zip(names)
        .map(|(output, (name, fault))| {
            let level = sends.get(&output.id).map(|level| format::decibels(*level));
            send(app, cue, output.id, level, (name, fault), name_room)
        });
    column![
        header(app, cue, &silent(app, &sends)),
        // Bar beside the fields, rather than over the last of each row
        scrollable(
            row(outputs)
                .spacing(ENTRY_SPACING)
                .wrap()
                .vertical_spacing(SPACING / 2.0)
        )
        .spacing(SPACING)
        .id(SENDS)
        .height(Fill)
    ]
    .spacing(SPACING)
    .into()
}

/// Returns the entry of one output: its name, the field of the level that
/// `cue` sends to it, which is `level` as text where it sends, and the unit
fn send<'a>(
    app: &App,
    cue: &Cue,
    output: OutputId,
    level: Option<String>,
    (name, fault): (String, Option<&Fault>),
    name_room: f32,
) -> Element<'a, Message> {
    let setting = Setting::Send(cue.id, output);
    let (field, shown) = typing(app, setting, "off", level.unwrap_or_default(), LEVEL_ROOM);
    let valid = rejection(app, setting, &shown).is_none();
    let field = styled(app, field, &shown, "off", LEVEL_ROOM, valid);
    let name = text(name)
        .width(fault.map_or(Length::Fixed(name_room), |_| Length::Shrink))
        .wrapping(Wrapping::None)
        .style(match fault {
            Some(fault) if fault.retried() => text::warning,
            Some(_) => text::danger,
            None => text::default,
        });
    row![name, field, text("dB")]
        .spacing(SPACING)
        .align_y(Center)
        .into()
}

/// Returns the title of the sends, the button that opens the outputs, and
/// the count of the outputs that `silent` lists, which the line about a
/// refused level replaces, since no room is left beside a field in a row of
/// its own
fn header<'a>(app: &App, cue: &Cue, silent: &[&Fault]) -> Element<'a, Message> {
    let told = match refusal(
        app,
        |setting| matches!(setting, Setting::Send(id, _) if *id == cue.id),
    ) {
        Some(reason) => refused(&reason),
        None => text(match silent.len() {
            0 => String::new(),
            count => format!("{} cannot play now", format::count(count, "output")),
        })
        .style(if silent.iter().all(|fault| fault.retried()) {
            text::warning
        } else {
            text::danger
        }),
    };
    row![
        text("Outputs"),
        button("Edit outputs").on_press(Message::ShowOutputs),
        told
    ]
    .spacing(SPACING)
    .align_y(Center)
    .into()
}
