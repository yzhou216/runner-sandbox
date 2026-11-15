// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Scrolling the cue list so that the selected cue stays in view, along
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

use crate::front::{App, Message, reveal, view};

/// Rows that the cue list keeps in view above and below the selected cue,
/// as far as the list has room, so that the cues next to it stay in sight,
/// such as those that come next as GO moves the selection down the list
const MARGIN_ROWS: f32 = 3.0;

impl App {
    /// The first cue snaps the list back to its start, as the window opened
    /// it, since the row itself begins a separator below the start.
    pub(super) fn revealed(self) -> (Self, Task<Message>) {
        // Scrolling moves the list away from the shape kept
        let app = self.remeasured();
        let task = match app.selected_row() {
            Some(0) => operation::snap_to(view::CUES, RelativeOffset::START),
            Some(_) => operate(Measure::default()).discard(),
            None => Task::none(),
        };
        let measure = app.measure_cues();
        (app, task.chain(measure))
    }
}

#[derive(Default)]
struct Measure {
    /// Top of the list content, how far the list has scrolled, and the
    /// height of the part in view
    list: Option<(f32, f32, f32)>,
    row: Option<Rectangle>,
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
        if id == Some(&Id::new(view::CUES)) {
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
        let Some(offset) = scrolling_to(span, scrolled, height) else {
            return Outcome::None;
        };
        let offset = AbsoluteOffset {
            x: None,
            y: Some(offset.max(0.0)),
        };
        Outcome::Chain(Box::new(scrollable::scroll_to(Id::new(view::CUES), offset)))
    }
}

/// The selected row, and the margin that the list keeps above and below it
#[derive(Clone, Copy)]
pub(super) struct Span {
    pub(super) top: f32,
    pub(super) height: f32,
    pub(super) margin: f32,
}

/// Returns where the cue list scrolls to in order to show the row of `span`
/// and its margin: as little as it can, but to the middle of the view when
/// the row lies far outside it, and `None` when it is in view.  The rule is
/// the one of Vim's `update_topline` and `scroll_cursor_bot` (src/move.c,
/// `:help scroll-cursor`), read in rows of the view with the margin as
/// `scrolloff`.  Above the view, a row centers once its top, less the margin,
/// lies half the view less one row above the top of the view, and at least
/// two rows.  Below it, the row centers once its bottom lies more than half
/// the view plus a row past the bottom of the view, whatever the margin.
pub(super) fn scrolling_to(span: Span, scrolled: f32, height: f32) -> Option<f32> {
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
