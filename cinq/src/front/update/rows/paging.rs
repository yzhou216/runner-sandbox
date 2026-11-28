// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Moving by half a page or a whole page, with the keys of Vim
//!
//! The update cannot tell how many rows the cue list shows, which depends
//! on the window and on how far the long names wrap, so a widget operation
//! measures the list as it is laid out, and the app keeps the measure for
//! the keys that follow at once.  The list then scrolls by the rows that the
//! selection moves, as Vim scrolls the window with its cursor, so that the
//! selection keeps its place on the screen.  The help page scrolls by the
//! same measure, in lines.

pub(super) mod kept;
pub(in crate::front::update) mod viewport;

use iced::{
    Task,
    advanced::widget::operate,
    widget::operation::{self, AbsoluteOffset},
};

use crate::front::{
    App, MAIN_SIZE, Message,
    message::Page,
    stepping::step,
    view::{self, line_height},
};
use kept::Measured;
use viewport::{Shape, Viewport};

/// Rows that a whole page leaves in view, as Vim leaves two lines
const OVERLAP: usize = 2;

impl App {
    /// Scrolls the cue list by a page and moves the selection to where
    /// that puts it.  A count before the keys of a half page sets how many
    /// rows it moves, which the next half pages keep, as the count sets
    /// Vim's `'scroll'`, while a count before the keys of a whole page
    /// counts the pages.
    pub(in crate::front::update) fn paged(
        self,
        page: Page,
        count: Option<usize>,
    ) -> (Self, Task<Message>) {
        let scroll = self.scrolling(page, count);
        let (rows, current, from) = (self.row_count(), self.row(), self.spot());
        Self { scroll, ..self }.landing(
            move |row| Message::Landed { from, row },
            move |shape| landing(page, count, scroll, shape, rows, current),
        )
    }

    /// Scrolls the help page by a page of the window it is open in.  The
    /// count of a half page sets its lines, shared with the cue list, and
    /// the count of a whole page is its pages.  The page is as tall as the
    /// main window while it is laid out too late to be measured, as it is
    /// when it has just opened.
    pub(in crate::front::update) fn help_paged(
        self,
        page: Page,
        count: Option<usize>,
    ) -> (Self, Task<Message>) {
        let scroll = self.scrolling(page, count);
        let by = move |view: f32| {
            operation::scroll_by(view::HELP, help_offset(page, count, scroll, view))
        };
        let (shown, window) = self.help_heights();
        let task = match shown {
            Some(view) => by(view),
            None => {
                let window = window.unwrap_or(MAIN_SIZE.height);
                operate(Viewport::of(view::HELP))
                    .collect()
                    .then(move |shapes| by(shapes.first().map_or(window, |shape| shape.view)))
            }
        };
        // The answer keeps the height in view
        let measure = operate(Viewport::of(view::HELP))
            .map(|shape| Message::Measured(Measured::Help(shape.view)));
        (Self { scroll, ..self }, task.chain(measure))
    }

    /// Returns the rows of a half page once `count` is typed before the keys
    /// of `page`, the count of a whole page being no such setting
    fn scrolling(&self, page: Page, count: Option<usize>) -> Option<usize> {
        count.filter(|_| page.is_half()).or(self.scroll)
    }
}

/// Returns how far a move by `page` scrolls the help page, `view` being the
/// height in view, `count` the count typed before the keys, and `scroll`
/// the lines of a half page that a count set
pub(in crate::front::update) fn help_offset(
    page: Page,
    count: Option<usize>,
    scroll: Option<usize>,
    view: f32,
) -> AbsoluteOffset {
    let lines = |lines: usize| lines as f32 * line_height();
    let distance = match (page.is_half(), scroll) {
        (true, Some(scroll)) => lines(scroll),
        (true, None) => view / 2.0,
        (false, _) => (view - lines(OVERLAP)).max(line_height()) * count.unwrap_or(1) as f32,
    };
    AbsoluteOffset {
        x: 0.0,
        y: page.sign() as f32 * distance,
    }
}

/// Where a page takes the cue list: how far it scrolls to, and the row,
/// counted from zero, that the selection moves to
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::front::update) struct Landing {
    pub offset: f32,
    pub row: usize,
}

/// Returns where a move by `page` takes a list of `cues` rows, `count`
/// being the count typed before the keys, `scroll` the rows of a half page
/// that an earlier count set, `shape` the list as it is laid out, and
/// `current` the selected row.  A half page moves the selection by the rows
/// that the list scrolls, so that it keeps its place on the screen, while a
/// whole page puts it on the first row in view going down, or the last going
/// up.  When the list cannot scroll as far as the page asks, the view stops
/// and the selection goes on to the last cue, or the first.
pub(in crate::front::update) fn landing(
    page: Page,
    count: Option<usize>,
    scroll: Option<usize>,
    shape: Shape,
    cues: usize,
    current: Option<usize>,
) -> Option<Landing> {
    let last = cues.checked_sub(1)?;
    let row_height = shape.content / cues as f32;
    let fit = rows_fitting(shape.view, shape.content, cues);
    let moved = rows(page, count, scroll, fit);
    let wanted = shape.scrolled + page.sign() as f32 * moved as f32 * row_height;
    let offset = wanted.clamp(0.0, (shape.content - shape.view).max(0.0));
    let (first_row, last_row) = Shape {
        scrolled: offset,
        ..shape
    }
    .rows_in_view(cues)?;
    let row = match page {
        Page::HalfDown | Page::HalfUp => {
            let by = isize::try_from(moved).unwrap_or(isize::MAX);
            step(current, page.sign() * by, cues)?
        }
        Page::WholeDown if wanted >= shape.content - shape.view => last,
        Page::WholeUp if wanted <= 0.0 => 0,
        Page::WholeDown => first_row.max(current.unwrap_or(0)),
        Page::WholeUp => last_row.min(current.unwrap_or(last)),
    };
    Some(Landing {
        offset,
        row: row.min(last),
    })
}

/// Returns the number of rows that a move by `page` scrolls, `fit` being
/// the rows that fit in the list, `count` the count typed before the keys,
/// and `scroll` the rows of a half page that an earlier count set
pub(in crate::front::update) fn rows(
    page: Page,
    count: Option<usize>,
    scroll: Option<usize>,
    fit: usize,
) -> usize {
    if page.is_half() {
        scroll.unwrap_or(fit / 2).max(1)
    } else {
        fit.saturating_sub(OVERLAP)
            .max(1)
            .saturating_mul(count.unwrap_or(1))
    }
}

/// Returns how many rows of `cues` fit in a view `view` tall over content
/// `content` tall, taking the average height of a row, since the rows that
/// a long file name wraps are taller than the rest
pub(in crate::front::update) fn rows_fitting(view: f32, content: f32, cues: usize) -> usize {
    if cues == 0 || content <= 0.0 {
        return 0;
    }
    (view / (content / cues as f32)) as usize
}

/// Measures the scrollable `id`, which holds the rows, as it is laid out,
/// then scrolls it to where `land` puts it for that shape and reports the row
/// that `land` puts the selection on with the message that `arrive` makes
fn measured(
    id: &'static str,
    arrive: impl Fn(usize) -> Message + Send + 'static,
    land: impl Fn(Shape) -> Option<Landing> + Send + 'static,
) -> Task<Message> {
    operate(Viewport::of(id)).then(move |shape| {
        land(shape).map_or_else(Task::none, |landing| {
            kept::scroll(id, landing.offset).chain(Task::done(arrive(landing.row)))
        })
    })
}
