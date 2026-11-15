// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Moving by half a page or a whole page, with the keys of Vim
//!
//! The update cannot tell how many rows the cue list shows, which depends
//! on the window and on how far the long names wrap, so a widget operation
//! measures the list as it is laid out.  The list then scrolls by the rows
//! that the selection moves, as Vim scrolls the window with its cursor, so
//! that the selection keeps its place on the screen.  The help page
//! scrolls by the same measure, in lines.

pub(super) mod viewport;

use iced::{
    Task,
    advanced::widget::operate,
    widget::operation::{self, AbsoluteOffset},
};

use crate::front::{
    App, Message,
    message::Page,
    step,
    view::{self, line_height},
};
use viewport::{Shape, Viewport};

/// Rows that a whole page leaves in view, as Vim leaves two lines
const OVERLAP: usize = 2;

impl App {
    /// Scrolls the cue list by a page and moves the selection to where
    /// that puts it.  A count before the keys of a half page sets how many
    /// rows it moves, which the next half pages keep, as the count sets
    /// Vim's `'scroll'`, while a count before the keys of a whole page
    /// counts the pages.
    pub(super) fn paged(self, page: Page, count: Option<usize>) -> (Self, Task<Message>) {
        let scroll = self.scrolling(page, count);
        let (cues, current) = (self.current().cues.len(), self.selected_row());
        let (list, from) = (self.list, self.selected);
        let task = measured(
            move |row| Message::Landed { list, from, row },
            move |shape| {
                sandbox!(
                    "page {page:?} count={count:?} view={} content={} cues={cues} fit={} moves={}",
                    shape.view,
                    shape.content,
                    rows_fitting(shape.view, shape.content, cues),
                    rows(
                        page,
                        count,
                        scroll,
                        rows_fitting(shape.view, shape.content, cues)
                    )
                );
                landing(page, count, scroll, shape, cues, current)
            },
        );
        (Self { scroll, ..self }, task)
    }

    /// Scrolls the help page by a page of the window it is open in.  The
    /// count of a half page sets its lines, shared with the cue list, and
    /// the count of a whole page is its pages.
    pub(super) fn help_paged(self, page: Page, count: Option<usize>) -> (Self, Task<Message>) {
        let scroll = self.scrolling(page, count);
        let task = operate(Viewport::of(view::HELP)).then(move |shape| {
            let lines = |lines: usize| lines as f32 * line_height();
            let distance = match (page.is_half(), scroll) {
                (true, Some(scroll)) => lines(scroll),
                (true, None) => shape.view / 2.0,
                (false, _) => {
                    let page = (shape.view - lines(OVERLAP)).max(line_height());
                    page * count.unwrap_or(1) as f32
                }
            };
            let offset = AbsoluteOffset {
                x: 0.0,
                y: page.sign() as f32 * distance,
            };
            operation::scroll_by(view::HELP, offset)
        });
        (Self { scroll, ..self }, task)
    }

    /// Returns the rows of a half page once `count` is typed before the keys
    /// of `page`, the count of a whole page being no such setting
    fn scrolling(&self, page: Page, count: Option<usize>) -> Option<usize> {
        count.filter(|_| page.is_half()).or(self.scroll)
    }
}

/// Where a page takes the cue list: how far it scrolls to, and the row,
/// counted from zero, that the selection moves to
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Landing {
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
pub(super) fn landing(
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
    let in_view = |row: f32| row as usize;
    let row = match page {
        Page::HalfDown | Page::HalfUp => {
            let by = isize::try_from(moved).unwrap_or(isize::MAX);
            step(current, page.sign() * by, cues)?
        }
        Page::WholeDown if wanted >= shape.content - shape.view => last,
        Page::WholeUp if wanted <= 0.0 => 0,
        Page::WholeDown => in_view((offset / row_height).ceil()).max(current.unwrap_or(0)),
        Page::WholeUp => {
            let bottom = in_view(((offset + shape.view) / row_height).floor());
            bottom.saturating_sub(1).min(current.unwrap_or(last))
        }
    };
    Some(Landing {
        offset,
        row: row.min(last),
    })
}

/// Returns the number of rows that a move by `page` scrolls, `fit` being
/// the rows that fit in the list, `count` the count typed before the keys,
/// and `scroll` the rows of a half page that an earlier count set
pub(super) fn rows(page: Page, count: Option<usize>, scroll: Option<usize>, fit: usize) -> usize {
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
pub(super) fn rows_fitting(view: f32, content: f32, cues: usize) -> usize {
    if cues == 0 || content <= 0.0 {
        return 0;
    }
    (view / (content / cues as f32)) as usize
}

/// Measures the cue list as it is laid out, then scrolls it to where `land`
/// puts it for that shape and reports the row that `land` puts the selection
/// on with the message that `arrive` makes
pub(super) fn measured(
    arrive: impl Fn(usize) -> Message + Send + 'static,
    land: impl Fn(Shape) -> Option<Landing> + Send + 'static,
) -> Task<Message> {
    operate(Viewport::of(view::CUES)).then(move |shape| {
        land(shape).map_or_else(Task::none, |landing| {
            sandbox!(
                "measured view={} content={} scrolled={} -> offset={} row={}",
                shape.view,
                shape.content,
                shape.scrolled,
                landing.offset,
                landing.row + 1
            );
            let offset = AbsoluteOffset {
                x: None,
                y: Some(landing.offset),
            };
            operation::scroll_to(view::CUES, offset).chain(Task::done(arrive(landing.row)))
        })
    })
}
