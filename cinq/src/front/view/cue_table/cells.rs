// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Columns of the cue list, and the cell that each draws for a cue

use std::rc::Rc;

use iced::{
    Element, Fill, Length, padding,
    widget::{container, mouse_area, radio, space, table, text, text::Wrapping},
};

use super::{
    ROW_PADDING,
    columns::{Column, Look, counted, shown_lines},
    layout::Layout,
    name,
};
use crate::{
    back::Cue,
    front::{App, Message, view::faults},
};

/// Returns `content` as text that breaks a word too wide for its column
/// rather than draws it over the next column
pub(super) fn broken<'a>(content: impl text::IntoFragment<'a>) -> text::Text<'a> {
    text(content).wrapping(Wrapping::WordOrGlyph)
}

pub(super) fn table_column<'a>(
    app: &'a App,
    column: Column,
    width: Option<f32>,
    titled: bool,
    counting: bool,
    layout: Rc<Layout<'a>>,
) -> table::Column<'a, 'a, &'a Cue, Message> {
    let header: Element<'a, Message> = if titled {
        text(column.title()).into()
    } else {
        space().into()
    };
    table::column(header, move |cue| cell(app, column, cue, counting, &layout))
        .width(width.map_or(Fill, Length::Fixed))
}

/// Returns the cell of `cue` in `column`, padded inside rather than by the
/// table, so that a click anywhere in it selects the cue, but on the mark of
/// a group, which flips it.  The cell of a name pads its parts itself, so that
/// the mark takes the height of the row.
fn cell<'a>(
    app: &'a App,
    column: Column,
    cue: &'a Cue,
    counting: bool,
    layout: &Layout,
) -> Element<'a, Message> {
    let groups = &layout.groups;
    let vertical = padding::vertical(ROW_PADDING);
    let (content, padded): (Element<'a, Message>, _) = match column {
        Column::Name => {
            let count = counted(&app.workspace, groups, cue).filter(|_| counting);
            let name = column.text(&app.workspace, groups, cue) + &count.unwrap_or_default();
            (
                name::cell(cue, name, layout.depth(cue), layout.gutter()),
                false,
            )
        }
        Column::Number => (
            radio(cue.number.clone(), cue.id, app.selected, Message::Select).into(),
            true,
        ),
        Column::State => {
            let failing = faults::failing(app);
            let look = Look::default()
                .with_next(layout.is_next(cue))
                .with_failing(&failing)
                .with_unreached(&app.problems.network_failed)
                .with_selected(app.selected == Some(cue.id));
            let lines = shown_lines(&app.workspace, groups, cue, look)
                .into_iter()
                .map(|(line, style)| broken(line).style(style).into());
            (iced::widget::column(lines).into(), true)
        }
        _ => (
            broken(column.text(&app.workspace, groups, cue)).into(),
            true,
        ),
    };
    let cell =
        container(content)
            .width(Fill)
            .padding(if padded { vertical } else { padding::all(0) });
    mouse_area(cell).on_press(Message::Select(cue.id)).into()
}
