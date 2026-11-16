// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The last measures of the rows in the main window and of the help page
//!
//! Measuring goes through the widget tree and so comes back a frame late.
//! Page keys that follow one another within that frame would each read the
//! list as it was before the earlier ones scrolled it, and move less than
//! their step.  The app therefore keeps what the latest measure found, lets
//! a landing use it at once and update it with where it scrolled to, and
//! measures again to correct it.  The first drawing of the window measures
//! the list, as does the first drawing after each change that makes the
//! measure out of date, such as a resize, so that a key finds one kept.  A
//! key that still finds nothing kept measures the list first.

use iced::{
    Task,
    advanced::widget::operate,
    widget::operation::{self, AbsoluteOffset},
};

use super::{
    Landing, measured,
    viewport::{Shape, Viewport},
};
use crate::front::{App, Message, update::Rows, view};

/// What a measure found, which the answer to it brings back
#[derive(Debug, Clone)]
pub enum Measured {
    /// Shape of the rows, as the measure numbered `serial` found it, for
    /// the rows and the number of them that it held
    Rows {
        serial: u64,
        rows: Rows,
        count: usize,
        shape: Shape,
    },
    /// Height of the help page in view
    Help(f32),
}

/// What the app keeps of the latest measures
#[derive(Debug, Default, Clone, Copy)]
pub struct Measures {
    rows: Option<Kept>,
    /// Height of the part of the help page in view
    help: Option<f32>,
    /// Height of the main window, to take the help page as tall as it
    /// while no measure of it is kept
    window: Option<f32>,
    /// Number of the latest measure that was asked for, which the answers
    /// to earlier ones fall short of
    serial: u64,
}

impl Measures {
    #[cfg(test)]
    pub(in crate::front) fn serial(self) -> u64 {
        self.serial
    }
}

/// Shape of the rows that were measured, for the rows and the number of them
/// that they held, since other rows or a change in their number makes the
/// measure out of date
#[derive(Debug, Clone, Copy)]
struct Kept {
    rows: Rows,
    count: usize,
    shape: Shape,
}

impl App {
    /// Lands as `land` tells for the rows in view: at once on the shape
    /// kept, which then scrolls to where the landing puts it, or after
    /// measuring the rows when nothing is kept.  `arrive` makes the message
    /// that selects the row of the landing.  Either way the rows are measured
    /// again after the scroll.
    pub(in crate::front::update) fn landing(
        self,
        arrive: impl Fn(usize) -> Message + Send + 'static,
        land: impl Fn(Shape) -> Option<Landing> + Send + 'static,
    ) -> (Self, Task<Message>) {
        let id = self.scrollable();
        let kept = self.measures.rows.filter(|kept| self.fits(*kept));
        let app = self.remeasured();
        let measure = app.measure_rows();
        let Some(kept) = kept else {
            return (app, measured(id, arrive, land).chain(measure));
        };
        let Some(landing) = land(kept.shape) else {
            return (app, Task::none());
        };
        let shape = Shape {
            scrolled: landing.offset,
            ..kept.shape
        };
        let (app, _) = Self {
            measures: Measures {
                rows: Some(Kept { shape, ..kept }),
                ..app.measures
            },
            ..app
        }
        .handled(arrive(landing.row));
        (app, scroll(id, landing.offset).chain(measure))
    }

    /// Returns whether a measure holds for the rows in view now
    fn fits(&self, kept: Kept) -> bool {
        (kept.rows, kept.count) == (self.rows(), self.row_count())
    }

    /// Returns the state with the kept measure of the rows out of date, and
    /// the answers to the measures asked for before it too
    pub(in crate::front::update) fn remeasured(self) -> Self {
        Self {
            measures: Measures {
                rows: None,
                serial: self.measures.serial.wrapping_add(1),
                ..self.measures
            },
            ..self
        }
    }

    /// Returns whether the rows are in view with no measure kept that fits
    /// them, which the next drawing of the main window measures
    pub(in crate::front::update) fn unmeasured(&self) -> bool {
        let fits = |kept: Kept| self.fits(kept);
        self.main.is_some() && !self.help && !self.measures.rows.is_some_and(fits)
    }

    /// Measures the rows, whose answer keeps the measure unless another
    /// measure has been asked for since
    pub(in crate::front::update) fn measure_rows(&self) -> Task<Message> {
        let (serial, rows, count) = (self.measures.serial, self.rows(), self.row_count());
        operate(Viewport::of(self.scrollable())).map(move |shape| {
            Message::Measured(Measured::Rows {
                serial,
                rows,
                count,
                shape,
            })
        })
    }

    pub(in crate::front::update) fn with_measure(
        self,
        serial: u64,
        rows: Rows,
        count: usize,
        shape: Shape,
    ) -> Self {
        if serial != self.measures.serial {
            return self;
        }
        Self {
            measures: Measures {
                rows: Some(Kept { rows, count, shape }),
                ..self.measures
            },
            ..self
        }
    }

    pub(in crate::front::update) fn with_help_measure(self, view: f32) -> Self {
        Self {
            measures: Measures {
                help: Some(view),
                ..self.measures
            },
            ..self
        }
    }

    /// Returns the height of the help page in view, and the height of the main
    /// window, each as far as it is known
    pub(in crate::front::update) fn help_heights(&self) -> (Option<f32>, Option<f32>) {
        (self.measures.help, self.measures.window)
    }

    /// Returns the state with the height of the main window it changed to,
    /// and no measure kept
    pub(in crate::front::update) fn with_window_height(self, height: f32) -> Self {
        let app = self.remeasured();
        Self {
            measures: Measures {
                window: Some(height),
                help: None,
                ..app.measures
            },
            ..app
        }
    }
}

/// Scrolls the scrollable of the rows to `offset`
pub(super) fn scroll(id: &'static str, offset: f32) -> Task<Message> {
    operation::scroll_to(
        id,
        AbsoluteOffset {
            x: None,
            y: Some(offset),
        },
    )
}

/// Measures the help page, whose answer keeps the height in view
pub(in crate::front::update) fn measure_help() -> Task<Message> {
    operate(Viewport::of(view::HELP)).map(|shape| Message::Measured(Measured::Help(shape.view)))
}
