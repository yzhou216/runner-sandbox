// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Scrolling the cue list a row at a time, and selecting the rows in its view,
//! with the keys of Vim
//!
//! Like paging, it measures the list as it is laid out, and takes the rows
//! to be as tall as the average, since the rows that a long name wraps are
//! taller than the rest.

use iced::Task;

use super::paging::{Landing, measured, viewport::Shape};
use crate::front::{App, Message, message::Height};

/// Slack for the rounding of a row height that is the quotient of a sum
const SLACK: f32 = 0.001;

impl App {
    /// Scrolls the cue list by `rows`, downwards when positive, and keeps
    /// the selection on its cue unless that leaves the view, when it moves to
    /// the nearest row in view, as the cursor does in Vim.
    pub(super) fn scrolled(self, rows: isize) -> (Self, Task<Message>) {
        let (cues, current) = (self.current().cues.len(), self.selected_row());
        let (list, from) = (self.list, self.selected);
        let task = measured(
            move |row| Message::Landed { list, from, row },
            move |shape| scroll(rows, shape, cues, current),
        );
        (self, task)
    }
}

/// Returns the first row, counted from zero, that is in view whole
fn first_in_view(shape: Shape, row_height: f32) -> usize {
    (shape.scrolled / row_height - SLACK).ceil().max(0.0) as usize
}

/// Returns the last row, counted from zero, that is in view whole, or the
/// first when not even one row fits
fn last_in_view(shape: Shape, row_height: f32) -> usize {
    let bottom = ((shape.scrolled + shape.view) / row_height + SLACK).floor() as usize;
    bottom
        .saturating_sub(1)
        .max(first_in_view(shape, row_height))
}

/// Returns where scrolling a list of `cues` rows by `rows` leaves the view
/// and the selection on row `current`, stopping at either end of the list
pub(super) fn scroll(
    rows: isize,
    shape: Shape,
    cues: usize,
    current: Option<usize>,
) -> Option<Landing> {
    let last = cues.checked_sub(1)?;
    let row_height = shape.content / cues as f32;
    let wanted = shape.scrolled + rows as f32 * row_height;
    let offset = wanted.clamp(0.0, (shape.content - shape.view).max(0.0));
    let moved = Shape {
        scrolled: offset,
        ..shape
    };
    let row = current.unwrap_or(0).clamp(
        first_in_view(moved, row_height),
        last_in_view(moved, row_height),
    );
    Some(Landing {
        offset,
        row: row.min(last),
    })
}

impl App {
    /// Selects the row at `height` in view, counted from the top for the
    /// first height and from the bottom for the last by `count`, as `H` and
    /// `L` do in Vim.  Selecting is a jump, as in Vim, and the view stays
    /// where it is.
    pub(super) fn in_view(self, height: Height, count: Option<usize>) -> (Self, Task<Message>) {
        let cues = self.current().cues.len();
        let (list, from) = (self.list, self.selected);
        let task = measured(
            move |row| Message::Reached { list, from, row },
            move |shape| in_view(height, count, shape, cues),
        );
        (self, task)
    }
}

/// Returns the row at `height` in view of a list of `cues` rows, with the
/// view left where it is.  A count that reaches past the rows in view stops
/// at the last of them, or the first, and the middle takes the upper of two
/// rows, as `M` does.
pub(super) fn in_view(
    height: Height,
    count: Option<usize>,
    shape: Shape,
    cues: usize,
) -> Option<Landing> {
    let last = cues.checked_sub(1)?;
    let row_height = shape.content / cues as f32;
    let first_row = first_in_view(shape, row_height).min(last);
    let last_row = last_in_view(shape, row_height).min(last);
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
    /// Scrolls the cue list to put the selected row at `height` in view,
    /// as `z t`, `z z`, and `z b` do in Vim, and leaves the selection on its
    /// cue.
    pub(super) fn placed(self, height: Height) -> (Self, Task<Message>) {
        let (cues, current) = (self.current().cues.len(), self.selected_row());
        let (list, from) = (self.list, self.selected);
        let task = measured(
            move |row| Message::Landed { list, from, row },
            move |shape| place(height, shape, cues, current?),
        );
        (self, task)
    }
}

/// Returns where the view of a list of `cues` rows must scroll to put row
/// `current` at `height`, stopping at either end of the list
pub(super) fn place(height: Height, shape: Shape, cues: usize, current: usize) -> Option<Landing> {
    let last = cues.checked_sub(1)?;
    let row = current.min(last);
    let row_height = shape.content / cues as f32;
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
