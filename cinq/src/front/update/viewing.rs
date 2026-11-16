// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Scrolling the rows a row at a time, and selecting the rows in view,
//! with the keys of Vim
//!
//! Like paging, it measures the rows as they are laid out, and takes the rows
//! to be as tall as the average, since the rows that a long name wraps are
//! taller than the rest.

use iced::Task;

use super::paging::{Landing, viewport::Shape};
use crate::front::{App, Message, message::Height};

impl App {
    /// Scrolls the rows by `by`, downwards when positive, and keeps
    /// the selection on its row unless that leaves the view, when it moves to
    /// the nearest row in view, as the cursor does in Vim.
    pub(super) fn scrolled(self, by: isize) -> (Self, Task<Message>) {
        let (rows, current, from) = (self.row_count(), self.row(), self.spot());
        self.landing(
            move |row| Message::Landed { from, row },
            move |shape| scroll(by, shape, rows, current),
        )
    }
}

/// Returns where scrolling a list of `rows` rows by `by` leaves the view
/// and the selection on row `current`, stopping at either end of the list
pub(super) fn scroll(
    by: isize,
    shape: Shape,
    rows: usize,
    current: Option<usize>,
) -> Option<Landing> {
    let row_height = shape.content / rows.max(1) as f32;
    let wanted = shape.scrolled + by as f32 * row_height;
    let offset = wanted.clamp(0.0, (shape.content - shape.view).max(0.0));
    let (first, last) = Shape {
        scrolled: offset,
        ..shape
    }
    .rows_in_view(rows)?;
    Some(Landing {
        offset,
        row: current.unwrap_or(0).clamp(first, last),
    })
}

impl App {
    /// Selects the row at `height` in view, counted from the top for the
    /// first height and from the bottom for the last by `count`, as `H` and
    /// `L` do in Vim.  Selecting is a jump, as in Vim, and the view stays
    /// where it is.
    pub(super) fn in_view(self, height: Height, count: Option<usize>) -> (Self, Task<Message>) {
        let (rows, from) = (self.row_count(), self.spot());
        self.landing(
            move |row| Message::Reached { from, row },
            move |shape| in_view(height, count, shape, rows),
        )
    }
}

/// Returns the row at `height` in view of a list of `rows` rows, with the
/// view left where it is.  A count that reaches past the rows in view stops
/// at the last of them, or the first, and the middle takes the upper of two
/// rows, as `M` does.
pub(super) fn in_view(
    height: Height,
    count: Option<usize>,
    shape: Shape,
    rows: usize,
) -> Option<Landing> {
    let (first_row, last_row) = shape.rows_in_view(rows)?;
    let from_edge = count.unwrap_or(1).saturating_sub(1);
    let row = match height {
        Height::Top => first_row.saturating_add(from_edge).min(last_row),
        Height::Middle => first_row + (last_row - first_row) / 2,
        Height::Bottom => last_row.saturating_sub(from_edge).max(first_row),
    };
    Some(Landing {
        offset: shape.scrolled,
        row,
    })
}
impl App {
    /// Scrolls the rows to put the selected row at `height` in view,
    /// as `z t`, `z z`, and `z b` do in Vim, and leaves the selection on its
    /// cue.  A count names the row, counted from one, that they select first,
    /// as `{count}zt` does, which is no jump.
    pub(super) fn placed(self, height: Height, count: Option<usize>) -> (Self, Task<Message>) {
        let app = match count {
            Some(row) => self.at_row(row),
            None => self,
        };
        let (rows, current, from) = (app.row_count(), app.row(), app.spot());

        app.landing(
            move |row| Message::Landed { from, row },
            move |shape| place(height, shape, rows, current?),
        )
    }
}

/// Returns where the view of a list of `rows` rows must scroll to put row
/// `current` at `height`, stopping at either end of the list
pub(super) fn place(height: Height, shape: Shape, rows: usize, current: usize) -> Option<Landing> {
    let last = rows.checked_sub(1)?;
    let row = current.min(last);
    let row_height = shape.content / rows as f32;
    let (top, bottom) = (row as f32 * row_height, (row + 1) as f32 * row_height);
    let wanted = match height {
        Height::Top => top,
        Height::Middle => (top + bottom - shape.view) / 2.0,
        Height::Bottom => bottom - shape.view,
    };
    Some(Landing {
        offset: wanted.clamp(0.0, (shape.content - shape.view).max(0.0)),
        row,
    })
}
