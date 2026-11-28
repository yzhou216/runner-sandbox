// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cue list, one row per cue that the list shows, under a header that stays in
//! view
//!
//! A collapsed group takes the row of the cues that it holds, as a closed fold
//! takes one line in Vim, so that the rows drawn are those of
//! [`Groups::shown`], in order.  The selection only drives the inspector,
//! while the State column marks the standby cue that GO fires as NEXT, on the
//! row of the collapsed group that hides it where one does.  Widths are
//! counted in columns of the monospace font, of which a wide character such
//! as one of Chinese takes two, so that the header, a table of its own above
//! the rows that scroll, lines up with them.

mod cells;
mod columns;
mod fitting;
mod layout;
mod name;
#[cfg(test)]
mod tests;

use std::rc::Rc;

use iced::{
    Element, Fill, padding,
    widget::{Radio, column, container, responsive, rule, scrollable, table, text},
};

use super::{CUES, SELECTED, faults, marked_row};
use crate::{
    back::{Cue, Groups, Workspace},
    front::{App, Message, keymap::Action},
};
use cells::table_column;
use columns::Column;
use fitting::fitting;
use layout::Layout;

/// Returns the texts of a cue that a search matches, which are those of the
/// columns of the list apart from the waits and the state, which change as
/// the show runs
pub(in crate::front) fn searchable(
    workspace: &Workspace,
    groups: &Groups,
    cue: &Cue,
) -> Vec<String> {
    use Column::{Action, Name, Number, Target, Then, Type};
    [Number, Type, Name, Target, Action, Then]
        .map(|column| column.text(workspace, groups, cue))
        .to_vec()
}

/// Width of the scroll bar, which the list leaves free at its right, so
/// that the bar covers no text and the rows keep the width of the header
/// with or without the bar
pub(super) const SCROLL_BAR: f32 = 10.0;

/// iced's own vertical padding of table cells
pub(super) const ROW_PADDING: f32 = 5.0;
/// iced's own horizontal padding of table cells, which both tables set,
/// since the widths of the columns count it
const CELL_PADDING: f32 = 10.0;
/// Width of the line between columns, which both tables set as well
const SEPARATOR: f32 = 1.0;

pub(super) const CELL_SPACE: f32 = 2.0 * CELL_PADDING + SEPARATOR;

pub(super) const RADIO: f32 = Radio::<Message>::DEFAULT_SIZE + Radio::<Message>::DEFAULT_SPACING;

// The widths below are in columns of the font

/// Fewest that the list keeps for a name before any column gives way
const NAME_LEAST: usize = 16;

/// Most that a name is given first call on, so that a name of up to as many
/// reads on one line however wide the window, and a very long name does not
/// keep the Target column narrow in a wide window
const NAME_MOST: usize = 48;

/// Most that the Target column takes while the room is short, past which a
/// long file name wraps
const TARGET_MOST: usize = 48;

/// Most that the Target column grows to when a name has all that it needs
const TARGET_GROWN: usize = 80;

/// Width that the Target column narrows to before any column gives way, enough
/// for a long path to wrap onto two lines, since a column that wraps onto more
/// makes every row taller than the name needs
const TARGET_ROOM: usize = 36;

/// Shift of a name to the right for each group that holds its cue
const INDENT: usize = 2;

/// Fewest that the State column holds, so that it keeps its width as cues run:
/// as many as `10:00.0 left`, the longest state of a cue whose sound is under
/// a hundred minutes.  It grows to the longest problem of a row, such as
/// `NO DESTINATION`.
const STATE_ROOM: usize = 12;

/// Width of the State column while a cue plays on a failed output, as many
/// as `OUTPUT FAILED` takes, which no running cue changes, so that a short
/// list keeps room for the Then column
const FAILED_ROOM: usize = 13;

/// Fewest that the State column keeps, as many as `DESTINATION`, the longest
/// word of its states, so that a narrow list never breaks a word of it
const STATE_LEAST: usize = 11;

pub(super) fn view(app: &App) -> Element<'_, Message> {
    responsive(move |size| list(app, size.width - SCROLL_BAR)).into()
}

fn list<'a>(app: &'a App, room: f32) -> Element<'a, Message> {
    let layout = Rc::new(Layout::of(app.current()));
    let fitting = fitting(app, &layout, room);
    // Where the Target column is gone, a collapsed group tells its count after its name
    let counting = fitting.iter().all(|&(column, _)| column != Column::Target);
    let table = |titled: bool, cues: Vec<&'a Cue>| {
        let layout = Rc::clone(&layout);
        let columns = fitting.iter().map(move |&(column, width)| {
            table_column(app, column, width, titled, counting, Rc::clone(&layout))
        });
        table::table(columns, cues)
            .padding_x(CELL_PADDING)
            .separator_x(SEPARATOR)
    };
    let header = table(true, Vec::new()).padding_y(ROW_PADDING);
    let rows = if app.current().cues.is_empty() {
        empty(app)
    } else {
        let selected = app.selected.and_then(|id| layout.row_of(id));
        let rows = table(false, layout.rows()).padding_y(0);
        marked_row(rows, fitting.len(), selected, SELECTED)
    };
    scrolling(header, rows, CUES)
}

/// Returns the `header` above the `rows`, which alone scroll in the
/// scrollable `id`, whose bar takes the width that both leave free
pub(super) fn scrolling<'a>(
    header: impl Into<Element<'a, Message>>,
    rows: impl Into<Element<'a, Message>>,
    id: &'static str,
) -> Element<'a, Message> {
    let free = padding::right(SCROLL_BAR);
    column![
        container(header).padding(free),
        rule::horizontal(1),
        scrollable(container(rows).padding(free))
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(SCROLL_BAR)
                    .scroller_width(SCROLL_BAR),
            ))
            .id(id)
            .height(Fill),
    ]
    .into()
}

fn empty(app: &App) -> Element<'_, Message> {
    let hint = match app.config.keymap.key(Action::AddBelow) {
        // Cues added here would not be those of the show that failed to open
        _ if app.problems.open_failed && app.problems.file.is_some() => {
            "The show could not be opened.  See the warning in the sidebar.".to_owned()
        }
        _ if !app.editable() => "No cues yet.  Leave show mode to add one.".to_owned(),
        Some(keys) => format!("No cues yet.  Press {keys} or Add cue to add one."),
        None => "No cues yet.  Press Add cue to add one.".to_owned(),
    };
    container(text(hint))
        .padding([ROW_PADDING, CELL_PADDING])
        .into()
}
