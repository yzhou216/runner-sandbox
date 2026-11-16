// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Status of the show, most important first, then at the right the keys of
//! a command typed so far, what the latest opening or saving told or else
//! the latest change to the show with the key that takes it back, the key
//! that cancels while keys wait or the search line is open, the keys that go
//! back and forward between the places jumped to while there is a place to
//! go to, the key that saves unsaved changes, the keys of GO, Panic, and the
//! help page, and, while a text field or a drop-down list has the keyboard,
//! the keys that go to the next field and that leave the field
//!
//! The bar keeps to one line.  When it is too narrow for all of it, the name
//! of the show is cut short at its end with an ellipsis, down to 16 columns,
//! and then the keys of going forward and back, of GO, of Panic, of saving,
//! of the help page, of the next field, and of leaving it give way in turn,
//! each one whole, and then what the latest change told gives way, at a word,
//! with an ellipsis, so that the status shows in full, while the keys typed so
//! far and the key of cancel always show in full.
//! While the outputs show, the bar names the keys that add and rename an
//! output, and the key that goes back always, since it is the way out, so
//! that the keys give way in another order: forward, add, rename, GO, Panic,
//! saving, back, the help page, the next field, and leaving a field.
//! An open search line takes the place of the status, and the hints give way
//! to its text as well.

mod fit;
mod hints;
pub(super) mod name;
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

const YIELDING: [Action; 10] = [
    Action::Forward,
    Action::Back,
    Action::Go,
    Action::Panic,
    Action::Save,
    Action::Help,
    Action::NextField,
    Action::Cancel,
    Action::CloseMenu,
    Action::Pick,
];

/// Hints in turn of giving way while the outputs show.  The keys that add
/// and rename an output go first, as they only teach, and the key that goes
/// back goes last but one, since it is the way out of the outputs, which
/// GO and Panic do not need, as the transport has buttons for them.
const YIELDING_IN_OUTPUTS: [Action; 12] = [
    Action::Forward,
    Action::AddBelow,
    Action::Edit,
    Action::Go,
    Action::Panic,
    Action::Save,
    Action::Back,
    Action::Help,
    Action::NextField,
    Action::Cancel,
    Action::CloseMenu,
    Action::Pick,
];

fn yielding(app: &App) -> &'static [Action] {
    if app.outputs_shown {
        &YIELDING_IN_OUTPUTS
    } else {
        &YIELDING
    }
}

/// Mark that opens the search line, or the one that opens it for a search that
/// goes up the list, as in Vim
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
            let shown = fit::shown(
                |told| hints::hints_told(app, told),
                hints::told(app),
                room.saturating_sub(taken),
                yielding(app),
            );
            (search_line(app, search), shown)
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
        .spacing(SPACING)
        .padding(SPACING)
        .align_y(Center)
        .into()
}

/// Returns the columns of text that fit in the bar of `width` pixels
fn columns(width: f32) -> usize {
    ((width - 3.0 * SPACING) / char_width()).floor() as usize
}

/// Columns that the search line keeps for its text, and for what it tells,
/// so that the hints never cover them
fn search_columns(text: &str, warned: bool, outputs: bool) -> usize {
    let told = if warned {
        no_match(outputs).width() + 1
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
            .padding(Padding::ZERO.right(SPACING))
        }))
        .align_y(Center);
    keep_keys(line_keys(line), &app.config.keymap)
}

/// Returns the status of the show with `name` for the name of its file
fn status(app: &App, name: String) -> Vec<text::Span<'_>> {
    let count = warnings(app);
    let mode = if app.editable() {
        "Edit mode"
    } else {
        "Show mode"
    };
    vec![
        span(name),
        span(app.show.mark()).color(theme::PALETTE.warning),
        span(LINE),
        span(mode).color_maybe((!app.editable()).then_some(theme::PALETTE.primary)),
        span(LINE),
        span(format::count(count, "warning"))
            .color_maybe((count > 0).then_some(theme::PALETTE.warning)),
    ]
}
