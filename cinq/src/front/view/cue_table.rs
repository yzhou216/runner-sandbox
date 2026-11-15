// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cue list, one row per cue, under a header that stays in view
//!
//! The selection only drives the inspector, while the State column marks
//! the standby cue that GO fires as NEXT.  Widths are counted in columns of
//! the monospace font, of which a wide character such as one of Chinese
//! takes two, so that the header, a table of its own above the rows that
//! scroll, lines up with them.

mod columns;

use iced::{
    Element, Fill, Length, padding,
    widget::{
        Radio, column, container, mouse_area, radio, responsive, rule, scrollable, space, table,
        text, text::Wrapping,
    },
};
use itertools::Itertools;
use strum::VariantArray;

use super::{CUES, SELECTED, char_width, giving_way, marked_row};
use crate::{
    back::Cue,
    front::{App, Message, keymap::Action},
};
use columns::{Column, state};

/// Width of the scroll bar, which the list leaves free at its right, so
/// that the bar covers no text and the rows keep the width of the header
/// with or without the bar
const SCROLL_BAR: f32 = 10.0;

/// iced's own vertical padding of table cells
const ROW_PADDING: f32 = 5.0;

/// iced's own horizontal padding of table cells, which both tables set,
/// since the widths of the columns count it
const CELL_PADDING: f32 = 10.0;

/// Width of the line between columns, which both tables set as well
const SEPARATOR: f32 = 1.0;

const CELL_SPACE: f32 = 2.0 * CELL_PADDING + SEPARATOR;

const RADIO: f32 = Radio::<Message>::DEFAULT_SIZE + Radio::<Message>::DEFAULT_SPACING;

/// Fewest columns of the font that the list keeps for a name before any
/// column gives way
const NAME_ROOM: usize = 16;

/// Most columns of the font that the Target column takes, past which a
/// long file name wraps
const TARGET_ROOM: usize = 24;

/// Columns of the font that the State column holds, so that the column
/// keeps its width as cues run: as many as `10:00.0 left` takes, the
/// longest state of a cue whose sound is under a hundred minutes
const STATE_ROOM: usize = 12;

pub(super) fn view(app: &App) -> Element<'_, Message> {
    responsive(move |size| list(app, size.width - SCROLL_BAR)).into()
}

fn list<'a>(app: &'a App, room: f32) -> Element<'a, Message> {
    let fitting = fitting(app, room);
    let table = |titled: bool, cues: &'a [Cue]| {
        let columns = fitting
            .iter()
            .map(move |&(column, width)| table_column(app, column, width, titled));
        table::table(columns, cues)
            .padding_x(CELL_PADDING)
            .separator_x(SEPARATOR)
    };
    let free = padding::right(SCROLL_BAR);
    let header = table(true, &[]).padding_y(ROW_PADDING);
    let cues = &app.current().cues;
    let rows = if cues.is_empty() {
        empty(app)
    } else {
        let rows = table(false, cues).padding_y(0);
        marked_row(rows, fitting.len(), app.selected_row(), SELECTED)
    };
    column![
        container(header).padding(free),
        rule::horizontal(1),
        scrollable(container(rows).padding(free))
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(SCROLL_BAR)
                    .scroller_width(SCROLL_BAR),
            ))
            .id(CUES)
            .height(Fill),
    ]
    .into()
}

fn empty(app: &App) -> Element<'_, Message> {
    let hint = match app.config.keymap.key(Action::AddBelow) {
        _ if !app.editable() => "No cues yet.  Leave show mode to add one.".to_owned(),
        Some(keys) => format!("No cues yet.  Press {keys} or Add cue to add one."),
        None => "No cues yet.  Press Add cue to add one.".to_owned(),
    };
    container(text(hint))
        .padding([ROW_PADDING, CELL_PADDING])
        .into()
}

/// Returns the columns that a list `room` wide shows, each with its width.
/// When showing them all leaves a name less than [`NAME_ROOM`] columns of
/// the font, the yielding columns give way in turn, those without text in
/// any row first, as they tell nothing, and the inspector shows them all.
fn fitting(app: &App, room: f32) -> Vec<(Column, Option<f32>)> {
    let list = app.current();
    let all: Vec<(Column, Option<f32>)> = Column::VARIANTS
        .iter()
        .map(|&column| (column, column.width(&app.workspace, list)))
        .collect();
    let yielding: Vec<Column> = Column::YIELDING
        .into_iter()
        .sorted_by_cached_key(|column| column.holds_text(&app.workspace, list))
        .collect();
    let name_room = |columns: &[(Column, Option<f32>)]| {
        room - columns
            .iter()
            .map(|(_, width)| width.unwrap_or(0.0) + CELL_SPACE)
            .sum::<f32>()
    };
    giving_way(
        &yielding,
        |gone| {
            all.iter()
                .copied()
                .filter(|(column, _)| !gone.contains(column))
                .collect()
        },
        |columns: &Vec<_>| name_room(columns) >= NAME_ROOM as f32 * char_width(),
    )
}

fn table_column<'a>(
    app: &'a App,
    column: Column,
    width: Option<f32>,
    titled: bool,
) -> table::Column<'a, 'a, &'a Cue, Message> {
    let header: Element<'a, Message> = if titled {
        text(column.title()).into()
    } else {
        space().into()
    };
    table::column(header, move |cue| cell(app, column, cue))
        .width(width.map_or(Fill, Length::Fixed))
}

/// Returns the cell of `cue` in `column`, padded inside rather than by the
/// table, so that a click anywhere in it selects the cue
fn cell<'a>(app: &'a App, column: Column, cue: &'a Cue) -> Element<'a, Message> {
    let list = app.current();
    let content: Element<'a, Message> = match column {
        Column::Number => radio(cue.number.clone(), cue.id, app.selected, Message::Select).into(),
        Column::Target => text(column.text(&app.workspace, list, cue))
            .wrapping(Wrapping::WordOrGlyph)
            .into(),
        Column::State => {
            let (state, style) = state(&app.workspace, list, cue);
            text(state).style(style).into()
        }
        _ => text(column.text(&app.workspace, list, cue)).into(),
    };
    let cell = container(content)
        .width(Fill)
        .padding(padding::vertical(ROW_PADDING));
    mouse_area(cell).on_press(Message::Select(cue.id)).into()
}
