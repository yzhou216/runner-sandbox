// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Outputs of the show, which the main window shows in place of a cue list
//! once the sidebar or the keys that step through the lists choose them
//!
//! Each row tells, in a few words, why an output cannot play where it
//! cannot: in yellow where cinq tries its device again by itself, and in red
//! where the output must change.  A click selects the output, which the
//! inspector then edits.

mod rows;
#[cfg(test)]
mod tests;

use iced::{Element, Fill, Length, widget::responsive};
use strum::{IntoStaticStr, VariantArray};
use unicode_width::UnicodeWidthStr;

use super::{
    char_width,
    cue_table::{CELL_SPACE, RADIO, SCROLL_BAR},
    giving_way, widest,
};
use crate::{
    back::{Device, Fault, Output, one_line},
    front::{
        App, Message,
        view::{faults, format},
    },
};

/// Columns that the names of outputs take at most before they wrap, so
/// that a long name leaves the device room
const NAME_ROOM: f32 = 24.0;

/// Columns that a name and a state keep once they have to give way, few enough
/// for a table beside the sidebar of a window of 512 pixels to keep the device
const SQUEEZED: f32 = 8.0;

/// Fewest columns that the Device column keeps, since a device told in a
/// few characters a line is of no use, and the other columns give way
/// before it does
const DEVICE_ROOM: f32 = 20.0;

/// Fewest columns that the Device column keeps once the State column has
/// gone, enough for `Default device` on one line, so that a narrow table
/// keeps the channels beside it for as long as it can
const DEVICE_LEAST: f32 = 14.0;

/// Most columns that the Device column asks for to keep a name on one line,
/// past which the name wraps as it does in a narrow table
const DEVICE_WIDEST: f32 = 32.0;

/// What gives way to the Device column, the first to go first
#[derive(PartialEq)]
enum Yield {
    /// The column goes, since the inspector and the sidebar tell the same of
    /// the selected output
    Drop(Column),
    /// The state wraps within [`SQUEEZED`] columns
    State,
    /// The name wraps within [`SQUEEZED`] columns
    Name,
}

/// Returns the texts of an output that a search matches, which are those of the
/// columns of the table apart from its number, which is no part of the output,
/// and its state, which changes with the device
pub(in crate::front) fn searchable(app: &App, output: &Output) -> Vec<String> {
    [
        Column::Name,
        Column::Device,
        Column::Channels,
        Column::NewCues,
    ]
    .map(|column| column.text(app, 0, output))
    .to_vec()
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
    fn text(self, app: &App, row: usize, output: &Output) -> String {
        match self {
            Column::Number => (row + 1).to_string(),
            Column::Name => one_line(&output.name),
            Column::Device => device(&output.device).to_owned(),
            Column::Channels => output.channels.to_string(),
            Column::NewCues => output.new_cues.map_or_else(
                || "off".to_owned(),
                |level| format!("{} dB", format::decibels(level)),
            ),
            Column::State => faults::fault(app, output.id).map_or_else(String::new, Fault::state),
        }
    }

    /// Returns the width of the widest text of the column, its title included
    fn widest(self, app: &App) -> f32 {
        let texts = app
            .workspace
            .outputs()
            .iter()
            .enumerate()
            .map(|(row, output)| self.text(app, row, output));
        widest(texts).max(<&str>::from(self).width()) as f32 * char_width()
    }

    /// Returns the width of the column: Device takes what the others leave,
    /// which fit their widest text, so that the state of an output keeps to
    /// one line, and names wrap past [`NAME_ROOM`], unless `gone` has them
    /// give way to the device.
    fn width(self, app: &App, gone: &[Yield]) -> Length {
        let widest = || self.widest(app);
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

/// Returns the width that the Device column needs to keep its widest text
/// on one line, at most [`DEVICE_WIDEST`] columns, so that a name too long
/// for any table does not drive the New cues column away
fn device_wanted(app: &App) -> f32 {
    Column::Device
        .widest(app)
        .clamp(DEVICE_ROOM * char_width(), DEVICE_WIDEST * char_width())
}

/// Returns the columns that show and their widths, as many as keep the
/// device on one line, or [`DEVICE_ROOM`] columns once the New cues column
/// has gone, in `room`, with the others giving way in turn, the State and
/// Channels columns last of all, or as few as they can, and then the name and
/// the state back at the width of their text for as much as the device does
/// not need
fn fitting(app: &App, room: f32) -> Vec<(Column, Length)> {
    let columns = giving_way(
        &[
            Yield::Drop(Column::NewCues),
            Yield::State,
            Yield::Name,
            Yield::Drop(Column::State),
            Yield::Drop(Column::Channels),
        ],
        |gone| {
            Column::VARIANTS
                .iter()
                .filter(|&&column| !gone.contains(&Yield::Drop(column)))
                .map(|&column| (column, column.width(app, gone)))
                .collect::<Vec<_>>()
        },
        |columns| {
            let shows = |shown: Column| columns.iter().any(|(column, _)| *column == shown);
            let wanted = match (shows(Column::NewCues), shows(Column::State)) {
                (true, _) => device_wanted(app),
                (false, true) => DEVICE_ROOM * char_width(),
                (false, false) => DEVICE_LEAST * char_width(),
            };
            device_room(columns, room) >= wanted
        },
    );
    let spare = device_room(&columns, room) - device_wanted(app);
    columns
        .into_iter()
        .scan(spare.max(0.0), |spare, (column, width)| {
            let grown = match (column, width, column.width(app, &[])) {
                (Column::Name | Column::State, Length::Fixed(now), Length::Fixed(whole)) => {
                    let grown = whole.min(now + *spare).max(now);
                    *spare -= grown - now;
                    Length::Fixed(grown)
                }
                _ => width,
            };
            Some((column, grown))
        })
        .collect()
}

/// Returns the width that the Device column has in `room` beside the other
/// `columns`
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
    responsive(move |size| rows::list(app, size.width - SCROLL_BAR)).into()
}

pub(super) fn device(device: &Device) -> &str {
    match device {
        Device::Default => "Default device",
        Device::Named { id, name } if name.is_empty() => id,
        Device::Named { name, .. } => name,
    }
}
