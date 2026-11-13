// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Outputs of the show, which the main window shows in place of a cue list
//! once the sidebar or the keys that step through the lists choose them
//!
//! Each row shows an output with the device and the channels it plays on,
//! the level at which new cues send to it, and why it cannot play where it
//! cannot.  A click selects the output, which the inspector then edits.

use iced::{
    Element, Fill, Length, padding,
    widget::{container, mouse_area, radio, scrollable, table, text, text::Wrapping},
};
use strum::{IntoStaticStr, VariantArray};
use unicode_width::UnicodeWidthStr;

use super::{
    char_width,
    cue_table::{RADIO, ROW_PADDING},
    marked_row,
};
use crate::{
    back::{Device, Output},
    front::{App, Message, view::format},
};

/// Identity of the selected output, as the selected cue has its own
const SELECTED: &str = "selected output";

#[derive(Clone, Copy, VariantArray, IntoStaticStr)]
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
            Column::State => app
                .audio_problems
                .iter()
                .find(|(id, _)| *id == output.id)
                .map_or_else(String::new, |(_, problem)| problem.clone()),
        }
    }

    /// Returns the width of the column.  Names, devices, and problems share
    /// what the others leave, which fit their widest text.
    fn width(self, app: &App) -> Length {
        let widest = || {
            app.workspace
                .outputs()
                .iter()
                .enumerate()
                .map(|(row, output)| self.text(app, row, output).width())
                .fold(<&str>::from(self).width(), usize::max) as f32
                * char_width()
        };
        match self {
            Column::Name | Column::Device | Column::State => Fill,
            Column::Number => Length::Fixed(widest() + RADIO),
            Column::Channels | Column::NewCues => Length::Fixed(widest()),
        }
    }
}

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let columns = Column::VARIANTS.iter().map(|&column| {
        let title = container(text(<&str>::from(column))).padding(padding::vertical(ROW_PADDING));
        table::column(title, move |(row, output): (usize, &Output)| {
            cell(app, column, row, output)
        })
        .width(column.width(app))
    });
    let outputs = table::table(columns, app.workspace.outputs().iter().enumerate()).padding_y(0);
    scrollable(marked_row(
        outputs,
        Column::VARIANTS.len(),
        app.output_row(),
        SELECTED,
    ))
    .height(Fill)
    .into()
}

/// Returns the cell of `output` in `column`, padded inside rather than by
/// the table, so that a click anywhere in it selects the output
fn cell<'a>(app: &'a App, column: Column, row: usize, output: &'a Output) -> Element<'a, Message> {
    let shown = column.text(app, row, output);
    let content: Element<'a, Message> = match column {
        Column::Number => radio(shown, output.id, app.output, Message::SelectOutput).into(),
        Column::State => text(shown).style(text::danger).into(),
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
