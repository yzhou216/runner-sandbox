// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Status of the show, most important first, and at the right the hints: the
//! keys typed so far, what the latest opening, saving, or change told, and
//! the keys that can act now
//!
//! Which keys can act depends on where the keyboard is, as [`hints::acts_here`]
//! tells.  While hint mode is on, the hints are the letters of a label typed
//! so far, the key that leaves the mode, and the keys of GO and Panic.
//!
//! The bar keeps to one line.  When it is too narrow for all of it, the name
//! of the show is cut short at its end with an ellipsis, down to 16 columns,
//! then the keys give way each one whole, in the order that [`order`] sets,
//! and then what the latest change told gives way, at a word, with an
//! ellipsis.  The status, the keys typed so far, and the key that cancels
//! always show in full.

mod fit;
mod hints;
mod name;
mod order;
#[cfg(test)]
mod tests;

use iced::{
    Center, Element, Fill, Padding,
    widget::{container, responsive, rich_text, row, span, text, text::Wrapping, text_input},
};
use unicode_width::UnicodeWidthStr;

use super::{
    LINE, SEARCH, SPACING, char_width, format, giving_way, keep_keys, line_height, line_keys,
    no_match, theme, warnings,
};
use crate::front::{App, Message, Search, keymap::Action};

/// Mark of the search line, `?` for a search that goes up the list, as in Vim
fn mark(backward: bool) -> &'static str {
    if backward { "?" } else { "/" }
}

/// Fewest columns that the search line keeps for itself, which the hints
/// give way to
const SEARCH_ROOM: usize = 24;

pub(super) fn view(app: &App) -> Element<'_, Message> {
    // Bar is one line of text high with the gap around it, since a
    // responsive widget fills all the height it is given
    let height = line_height() + 2.0 * SPACING;
    container(responsive(move |size| bar(app, size.width)))
        .height(height)
        .into()
}

/// Hints grow to the left, so that the keys typed so far come and go
/// without moving the rest.  An open search line takes the place of the
/// status, as Vim's command line does, and the hints give way to its text.
fn bar(app: &App, width: f32) -> Element<'_, Message> {
    let room = columns(width);
    let (left, shown) = match &app.search {
        Some(search) => {
            let taken = search_columns(&search.text, search.warned, app.outputs_shown);
            (
                search_line(app, search),
                hints_in(app, room.saturating_sub(taken)),
            )
        }
        None => {
            let (name, shown) = name::shown(app, room);
            let status = container(rich_text(status(app, name)).wrapping(Wrapping::None))
                .width(Fill)
                .clip(true);
            (status.into(), shown)
        }
    };
    row![left, text(shown).wrapping(Wrapping::None)]
        .spacing(gap())
        .padding(SPACING)
        .align_y(Center)
        .into()
}

fn hints_in(app: &App, room: usize) -> String {
    let told = hints::notices::told(app);
    // A cue gone silent is what the operator cannot hear, so that its notice
    // outlasts what Enter does, which only teaches
    let silent = told.as_deref().is_some_and(format::silences);
    let outlasting = order::outlasting(app)
        .into_iter()
        .filter(|action| !(silent && *action == Action::Pick))
        .collect::<Vec<_>>();
    fit::shown(
        |told| hints::hints_told(app, told),
        told,
        room,
        order::yielding(app),
        &outlasting,
    )
}

/// Columns between the status and the hints, the fewest that keep a count
/// and a notice on a tight bar from reading as one phrase
const GAP: usize = 2;

fn gap() -> f32 {
    GAP as f32 * char_width()
}

fn columns(width: f32) -> usize {
    ((width - 2.0 * SPACING - gap()) / char_width()).floor() as usize
}

/// Columns that the search line keeps for its text and its note, which the
/// hints never cover
fn search_columns(text: &str, warned: bool, outputs: bool) -> usize {
    let told = if warned {
        no_match(outputs).width() + GAP
    } else {
        0
    };
    (mark(false).width() + text.width() + told).max(SEARCH_ROOM)
}

/// Search line: its mark, the text typed, and a note that no cue matches
/// for a while after a search finds none.  It keeps the keys typed into
/// it, as the fields of the inspector do.
fn search_line<'a>(app: &'a App, search: &'a Search) -> Element<'a, Message> {
    let field = text_input("", &search.text)
        .id(SEARCH)
        .on_input(Message::Searching)
        .on_submit(Message::SearchAccepted)
        .padding(0)
        .style(theme::line);
    let line = row![text(mark(search.backward)), field]
        .push(search.warned.then(|| {
            container(
                text(no_match(app.outputs_shown))
                    .color(theme::PALETTE.warning)
                    .wrapping(Wrapping::None),
            )
            .padding(Padding::ZERO.left(gap()).right(SPACING))
        }))
        .align_y(Center);
    keep_keys(line_keys(line), &app.config.keymap)
}

/// Returns the status of the show under `name`, with the number of its
/// warnings only while it has any, since silence tells that nothing is wrong
fn status(app: &App, name: String) -> Vec<text::Span<'_>> {
    let count = warnings(app);
    let mode = if app.editable() {
        "Edit mode"
    } else {
        "Show mode"
    };
    let warned = (count > 0).then(|| {
        [
            span(LINE),
            span(format::count(count, "warning")).color(theme::PALETTE.warning),
        ]
    });
    [
        span(name),
        span(app.show.mark()).color(theme::PALETTE.warning),
        span(LINE),
        span(mode).color_maybe((!app.editable()).then_some(theme::PALETTE.primary)),
    ]
    .into_iter()
    .chain(warned.into_iter().flatten())
    .collect()
}
