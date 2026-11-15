// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Outputs of the show, which the main window shows in place of a cue list
//! once the sidebar or the keys that step through the lists choose them
//!
//! Each row shows an output with the device and the channels it plays on,
//! the level at which new cues send to it, and, in a few words, why it
//! cannot play where it cannot: in yellow where cinq tries its device again
//! by itself, and in red where the output must change.  A click selects the
//! output, which the inspector then edits.

#[cfg(test)]
mod tests;

use iced::{
    Element, Fill, Length, padding,
    widget::{
        column, container, mouse_area, radio, responsive, rule, scrollable, space, table, text,
        text::Wrapping,
    },
};
use strum::{IntoStaticStr, VariantArray};
use unicode_width::UnicodeWidthStr;

use super::{
    OUTPUTS, SELECTED, char_width,
    cue_table::{CELL_SPACE, RADIO, ROW_PADDING, SCROLL_BAR},
    giving_way, marked_row,
};
use crate::{
    back::{Device, Fault, Output},
    front::{
        App, Message,
        view::{faults, format},
    },
};

/// Columns that the names of outputs take at most before they wrap, so
/// that a long name leaves the device room
const NAME_ROOM: f32 = 24.0;

/// Columns that a name and a state keep once they have to give way
const SQUEEZED: f32 = 12.0;

/// Fewest columns that the Device column keeps, since a device told in a
/// few characters a line is of no use, and the other columns give way
/// before it does
const DEVICE_ROOM: f32 = 20.0;

/// What gives way to the Device column, the first to go first
#[derive(Clone, Copy, PartialEq)]
enum Yield {
    /// The New cues column goes
    NewCues,
    /// The state wraps within [`SQUEEZED`] columns
    State,
    /// The name wraps within [`SQUEEZED`] columns
    Name,
}

#[derive(Clone, Copy, PartialEq, VariantArray, IntoStaticStr)]
enum Column {
    #[strum(serialize = "#")]
    Number,
    Name,
    Device,
    Channels,
    #[strum(serialize = "New cues")]
    NewCues,
    State,
}

impl Column {
    /// Returns what the column shows of the output at index `row`
    fn text(self, app: &App, row: usize, output: &Output) -> String {
        match self {
            Column::Number => (row + 1).to_string(),
            Column::Name => output.name.clone(),
            Column::Device => device(&output.device).to_owned(),
            Column::Channels => output.channels.to_string(),
            Column::NewCues => output.new_cues.map_or_else(
                || "off".to_owned(),
                |level| format!("{} dB", format::decibels(level)),
            ),
            Column::State => faults::fault(app, output.id).map_or_else(String::new, Fault::state),
        }
    }

    /// Returns the width of the column.  Devices take what the others
    /// leave, which fit their widest text, so that the state of an output
    /// keeps to one line, and names wrap past [`NAME_ROOM`], unless `gone`
    /// has them give way to the device.
    fn width(self, app: &App, gone: &[Yield]) -> Length {
        let widest = || {
            app.workspace
                .outputs()
                .iter()
                .enumerate()
                .map(|(row, output)| self.text(app, row, output).width())
                .fold(<&str>::from(self).width(), usize::max) as f32
                * char_width()
        };
        let squeezed = |gives: Yield, room: f32| {
            let room = if gone.contains(&gives) {
                SQUEEZED
            } else {
                room
            };
            Length::Fixed(widest().min(room * char_width()))
        };
        match self {
            Column::Device => Fill,
            Column::Name => squeezed(Yield::Name, NAME_ROOM),
            Column::Number => Length::Fixed(widest() + RADIO),
            Column::State => squeezed(Yield::State, f32::INFINITY),
            Column::Channels | Column::NewCues => Length::Fixed(widest()),
        }
    }
}

/// Returns the columns that show and their widths, as many as keep the
/// device [`DEVICE_ROOM`] columns in `room`, with the others giving way in
/// turn, or as few as they can
fn fitting(app: &App, room: f32) -> Vec<(Column, Length)> {
    giving_way(
        &[Yield::NewCues, Yield::State, Yield::Name],
        |gone| {
            Column::VARIANTS
                .iter()
                .filter(|&&column| {
                    !(matches!(column, Column::NewCues) && gone.contains(&Yield::NewCues))
                })
                .map(|&column| (column, column.width(app, gone)))
                .collect::<Vec<_>>()
        },
        |columns| device_room(columns, room) >= DEVICE_ROOM * char_width(),
    )
}

/// Returns the width that the Device column has in `room` beside the other
/// `columns`, and the space between the columns
fn device_room(columns: &[(Column, Length)], room: f32) -> f32 {
    let fixed: f32 = columns
        .iter()
        .filter_map(|(_, width)| match width {
            Length::Fixed(width) => Some(*width),
            _ => None,
        })
        .sum();
    room - fixed - CELL_SPACE * columns.len() as f32
}

pub(super) fn view(app: &App) -> Element<'_, Message> {
    responsive(move |size| list(app, size.width - SCROLL_BAR)).into()
}

/// Returns the outputs with the titles of their columns above, which stay in
/// place as the rows scroll, so that the rows are all that the scrollable
/// holds, as in the cue list, and the keys that scroll and measure rows count
/// them alone
fn list<'a>(app: &'a App, room: f32) -> Element<'a, Message> {
    let shown = fitting(app, room);
    let table = |titled: bool, outputs: &'a [Output]| {
        let columns = shown.iter().copied().map(|(column, width)| {
            let title: Element<'_, Message> = if titled {
                container(text(<&str>::from(column)))
                    .padding(padding::vertical(ROW_PADDING))
                    .into()
            } else {
                space().into()
            };
            table::column(title, move |(row, output): (usize, &Output)| {
                cell(app, column, row, output)
            })
            .width(width)
        });
        table::table(columns, outputs.iter().enumerate()).padding_y(0)
    };
    let free = padding::right(SCROLL_BAR);
    let rows = marked_row(
        table(false, app.workspace.outputs()),
        shown.len(),
        app.output_row(),
        SELECTED,
    );
    column![
        container(table(true, &[])).padding(free),
        rule::horizontal(1),
        scrollable(container(rows).padding(free))
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(SCROLL_BAR)
                    .scroller_width(SCROLL_BAR),
            ))
            .id(OUTPUTS)
            .height(Fill),
    ]
    .into()
}

/// Returns the cell of `output` in `column`, padded inside rather than by
/// the table, so that a click anywhere in it selects the output
fn cell<'a>(app: &'a App, column: Column, row: usize, output: &'a Output) -> Element<'a, Message> {
    let shown = column.text(app, row, output);
    let content: Element<'a, Message> = match column {
        Column::Number => radio(shown, output.id, app.output, Message::SelectOutput).into(),
        Column::State => {
            let retried = faults::fault(app, output.id).is_some_and(Fault::retried);
            text(shown)
                .style(if retried { text::warning } else { text::danger })
                .into()
        }
        _ => text(shown).wrapping(Wrapping::WordOrGlyph).into(),
    };
    let cell = container(content)
        .width(Fill)
        .padding(padding::vertical(ROW_PADDING));
    mouse_area(cell)
        .on_press(Message::SelectOutput(output.id))
        .into()
}

/// Returns the name that an output goes by for its device
pub(super) fn device(device: &Device) -> &str {
    match device {
        Device::Default => "Default device",
        Device::Named { id, name } if name.is_empty() => id,
        Device::Named { name, .. } => name,
    }
}
