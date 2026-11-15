// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Moving by half a page or a whole page, with the keys of Vim
//!
//! The update cannot tell how many rows the cue list shows, which depends
//! on the window and on how far the long names and file names wrap, so a
//! widget operation measures the list as it is laid out, and the rows that
//! fit turn the key into an ordinary step of the selection, which scrolls
//! into view as every step does.  The help page scrolls by the same
//! measure, in lines.

use iced::{
    Rectangle, Task, Vector,
    advanced::widget::{
        Id, operate,
        operation::{Operation, Outcome, Scrollable},
    },
    widget::operation::{self, AbsoluteOffset},
};

use crate::front::{
    App, Message,
    message::Page,
    view::{self, line_height},
};

impl App {
    /// Steps the selection by a page of the rows that fit in the cue list.
    /// A count before the keys of a half page sets how many rows it moves,
    /// which the next half pages keep, as the count sets Vim's `'scroll'`,
    /// while a count before the keys of a whole page counts the pages.
    pub(super) fn paged(self, page: Page, count: Option<usize>) -> (Self, Task<Message>) {
        let scroll = count.filter(|_| page.is_half()).or(self.scroll);
        let cues = self.current().cues.len();
        let task = operate(Viewport::of(view::CUES)).map(move |(view, content)| {
            let fit = rows_fitting(view, content, cues);
            let moved = isize::try_from(rows(page, count, scroll, fit)).unwrap_or(isize::MAX);
            eprintln!(
                "SANDBOX page {page:?} count={count:?} view={view} content={content} cues={cues} fit={fit} moves={moved}"
            );
            Message::Step(page.sign() * moved)
        });
        (Self { scroll, ..self }, task)
    }
}

/// Returns the number of rows that a move by `page` takes, `fit` being the
/// rows that fit in the list, `count` the count typed before the keys, and
/// `scroll` the rows of a half page that an earlier count set
pub(super) fn rows(page: Page, count: Option<usize>, scroll: Option<usize>, fit: usize) -> usize {
    if page.is_half() {
        scroll.unwrap_or(fit / 2).max(1)
    } else {
        fit.max(1).saturating_mul(count.unwrap_or(1))
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

/// Scrolls the help page by half a page or a whole page of the window it
/// is open in.  The count of a half page is its lines, for that press only,
/// and the count of a whole page is its pages.
pub(super) fn help_paged(page: Page, count: Option<usize>) -> Task<Message> {
    operate(Viewport::of(view::HELP)).then(move |(view, _)| {
        let lines = |count: usize| count as f32 * line_height();
        let distance = match (page.is_half(), count) {
            (true, Some(count)) => lines(count),
            (true, None) => view / 2.0,
            (false, count) => view * count.unwrap_or(1) as f32,
        };
        let offset = AbsoluteOffset {
            x: 0.0,
            y: page.sign() as f32 * distance,
        };
        operation::scroll_by(view::HELP, offset)
    })
}

/// Finds the height of a scrollable as it is in view, and of its content
struct Viewport {
    id: &'static str,
    found: Option<(f32, f32)>,
}

impl Viewport {
    fn of(id: &'static str) -> Self {
        Self { id, found: None }
    }
}

impl Operation<(f32, f32)> for Viewport {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<(f32, f32)>)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        _translation: Vector,
        _state: &mut dyn Scrollable,
    ) {
        if id == Some(&Id::new(self.id)) {
            self.found = Some((bounds.height, content_bounds.height));
        }
    }

    fn finish(&self) -> Outcome<(f32, f32)> {
        self.found.map_or(Outcome::None, Outcome::Some)
    }
}
