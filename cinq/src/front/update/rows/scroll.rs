// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Scrolling the rows so that the selected row stays in view, along
//! with a few rows around it
//!
//! The list measures the selected row as it is laid out, so a row that a
//! long name wraps onto two lines scrolls into view as surely as any other.
//! Measuring and scrolling happen in one operation, which iced carries out
//! before it draws the next frame.

use iced::{
    Rectangle, Task, Vector,
    advanced::widget::{
        Id, operate,
        operation::{Operation, Outcome, Scrollable, scrollable},
    },
    widget::operation::{self, AbsoluteOffset, RelativeOffset},
};

use crate::front::{App, Message, stepping::reveal, view};

/// Rows that the cue list keeps in view above and below the selected cue,
/// as far as the list has room, so that the cues next to it stay in sight,
/// such as those that come next as GO moves the selection down the list
const MARGIN_ROWS: f32 = 3.0;

impl App {
    /// The first cue snaps the list back to its start, as the window opened
    /// it, since the row itself begins a separator below the start.  A cue
    /// that a collapsed group hides comes out of it first, since the
    /// selection is never hidden.
    pub(in crate::front::update) fn revealed(self) -> (Self, Task<Message>) {
        // Scrolling moves the list away from the shape kept
        let app = self.with_selection_unhidden().remeasured();
        let (id, row) = (app.scrollable(), app.row());
        let task = match row {
            Some(0) => operation::snap_to(id, RelativeOffset::START),
            Some(_) => operate(Measure::of(id)).discard(),
            None => Task::none(),
        };
        let measure = app.measure_rows();
        (app, task.chain(measure))
    }

    /// Applies `change`, and brings the selection into view only if the
    /// change moved it, so that a change that leaves the selection where it
    /// was leaves the list where it was too
    pub(in crate::front::update) fn revealing_a_move(
        self,
        change: impl FnOnce(Self) -> Self,
    ) -> (Self, Task<Message>) {
        let selected = self.selected;
        let app = change(self);
        if app.selected == selected {
            (app, Task::none())
        } else {
            app.revealed()
        }
    }
}

struct Measure {
    scrollable: &'static str,
    /// Top of the list content, how far the list has scrolled, and the
    /// height of the part in view
    list: Option<(f32, f32, f32)>,
    row: Option<Rectangle>,
}

impl Measure {
    fn of(scrollable: &'static str) -> Self {
        Self {
            scrollable,
            list: None,
            row: None,
        }
    }
}

impl Operation for Measure {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn Scrollable,
    ) {
        if id == Some(&Id::new(self.scrollable)) {
            self.list = Some((content_bounds.y, translation.y, bounds.height));
        }
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if id == Some(&Id::new(view::SELECTED)) {
            self.row = Some(bounds);
        }
    }

    fn finish(&self) -> Outcome<()> {
        let (Some((origin, scrolled, height)), Some(row)) = (self.list, self.row) else {
            return Outcome::None;
        };
        let margin = (MARGIN_ROWS * row.height).min(((height - row.height) / 2.0).max(0.0));
        let span = Span {
            top: row.y - origin,
            height: row.height,
            margin,
        };
        scrolled_to(self.scrollable, scrolling_to(span, scrolled, height))
    }
}

/// Returns the operation that scrolls the scrollable to `offset`, or to its
/// start when `offset` lies above it, and that does nothing without an offset
pub(in crate::front::update) fn scrolled_to(
    scrollable: &'static str,
    offset: Option<f32>,
) -> Outcome<()> {
    offset.map_or(Outcome::None, |offset| {
        let to = AbsoluteOffset {
            x: None,
            y: Some(offset.max(0.0)),
        };
        Outcome::Chain(Box::new(scrollable::scroll_to(Id::new(scrollable), to)))
    })
}

/// Selected row, and the margin that the list keeps above and below it
#[derive(Clone, Copy)]
pub(in crate::front::update) struct Span {
    pub(in crate::front::update) top: f32,
    pub(in crate::front::update) height: f32,
    pub(in crate::front::update) margin: f32,
}

/// Returns where the rows scroll to in order to show the row of `span`
/// and its margin: as little as it can, but to the middle of the view when
/// the row lies far outside it, and `None` when it is in view.  The rule is
/// the one of Vim's `update_topline` and `scroll_cursor_bot` (src/move.c,
/// `:help scroll-cursor`), read in rows of the view with the margin as
/// `scrolloff`.  Above the view, a row centers once its top, less the margin,
/// lies half the view less one row above the top of the view, and at least
/// two rows.  Below it, the row centers once its bottom lies more than half
/// the view plus a row past the bottom of the view, whatever the margin.
pub(in crate::front::update) fn scrolling_to(
    span: Span,
    scrolled: f32,
    height: f32,
) -> Option<f32> {
    let Span {
        top,
        height: row,
        margin,
    } = span;
    let (top, bottom) = (top - margin, top + row + margin);
    let half = ((height / row / 2.0).floor() - 1.0).max(2.0) * row;
    let far_above = scrolled - top >= half;
    let far_below = bottom - margin - (scrolled + height) > height / 2.0 + row;
    if far_above || far_below {
        Some((top + bottom - height) / 2.0)
    } else {
        reveal(top, bottom, scrolled, height)
    }
}
