// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rows of the outputs, with the titles of their columns above them

use iced::{
    Element, Fill, padding,
    widget::{container, mouse_area, radio, space, table, text, text::Wrapping},
};

use super::{Column, fitting};
use crate::{
    back::{Fault, Output},
    front::{
        App, Message,
        view::{
            OUTPUTS, SELECTED,
            cue_table::{ROW_PADDING, scrolling},
            faults, marked_row, theme,
        },
    },
};

/// Returns the outputs with the titles of their columns above, which stay in
/// place as the rows scroll, so that the rows are all that the scrollable
/// holds, as in the cue list, and the keys that scroll and measure rows count
/// them alone
pub(super) fn list<'a>(app: &'a App, room: f32) -> Element<'a, Message> {
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
    let rows = marked_row(
        table(false, app.workspace.outputs()),
        shown.len(),
        app.output_row(),
        SELECTED,
    );
    scrolling(table(true, &[]), rows, OUTPUTS)
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
                .style(if retried {
                    text::warning
                } else {
                    theme::danger(app.output == Some(output.id))
                })
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
