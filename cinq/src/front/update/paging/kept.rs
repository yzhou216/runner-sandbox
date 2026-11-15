// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The last measures of the cue list and of the help page
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
use crate::front::{App, Message, view};

/// What a measure found, which the answer to it brings back
#[derive(Debug, Clone)]
pub enum Measured {
    /// Shape of the cue list, as the measure numbered `serial` found it, for
    /// the list and the number of cues that it held
    Cues {
        serial: u64,
        list: usize,
        cues: usize,
        shape: Shape,
    },
    /// Height of the help page in view
    Help(f32),
}

/// What the app keeps of the latest measures
#[derive(Debug, Default, Clone, Copy)]
pub struct Measures {
    cues: Option<Kept>,
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

/// Shape of the cue list that was measured, for the list and the number of
/// cues that it held, since another list or a change in the number of its
/// rows makes the measure out of date
#[derive(Debug, Clone, Copy)]
struct Kept {
    list: usize,
    cues: usize,
    shape: Shape,
}

impl App {
    /// Lands as `land` tells for the cue list: at once on the shape kept,
    /// which then scrolls to where the landing puts it, or after measuring
    /// the list when nothing is kept.  `arrive` makes the message that
    /// selects the row of the landing.  Either way the list is measured
    /// again after the scroll.
    pub(in crate::front::update) fn landing(
        self,
        arrive: impl Fn(usize) -> Message + Send + 'static,
        land: impl Fn(Shape) -> Option<Landing> + Send + 'static,
    ) -> (Self, Task<Message>) {
        let land = move |shape: Shape| {
            let landing = land(shape);
            if let Some(landing) = &landing {
                sandbox!(
                    "measured view={} content={} scrolled={} -> offset={} row={}",
                    shape.view,
                    shape.content,
                    shape.scrolled,
                    landing.offset,
                    landing.row + 1
                );
            }
            landing
        };
        let (list, cues) = (self.list, self.current().cues.len());
        let kept = self
            .measures
            .cues
            .filter(|kept| (kept.list, kept.cues) == (list, cues));
        let app = self.remeasured();
        let measure = app.measure_cues();
        let Some(kept) = kept else {
            return (app, measured(arrive, land).chain(measure));
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
                cues: Some(Kept { shape, ..kept }),
                ..app.measures
            },
            ..app
        }
        .handled(arrive(landing.row));
        (app, scroll(landing.offset).chain(measure))
    }

    /// Returns the state with the kept measure of the cue list out of date,
    /// and the answers to the measures asked for before it too
    pub(in crate::front::update) fn remeasured(self) -> Self {
        Self {
            measures: Measures {
                cues: None,
                serial: self.measures.serial.wrapping_add(1),
                ..self.measures
            },
            ..self
        }
    }

    /// Returns whether the cue list is in view with no measure kept that fits
    /// it, which the next drawing of the main window measures
    pub(in crate::front::update) fn unmeasured(&self) -> bool {
        let fits = |kept: Kept| (kept.list, kept.cues) == (self.list, self.current().cues.len());
        self.main.is_some() && !self.help && !self.measures.cues.is_some_and(fits)
    }

    /// Measures the cue list, whose answer keeps the measure unless another
    /// measure has been asked for since
    pub(in crate::front::update) fn measure_cues(&self) -> Task<Message> {
        let (serial, list, cues) = (self.measures.serial, self.list, self.current().cues.len());
        operate(Viewport::of(view::CUES)).map(move |shape| {
            Message::Measured(Measured::Cues {
                serial,
                list,
                cues,
                shape,
            })
        })
    }

    pub(in crate::front::update) fn with_measure(
        self,
        serial: u64,
        list: usize,
        cues: usize,
        shape: Shape,
    ) -> Self {
        sandbox!(
            "measure view={} content={} scrolled={}",
            shape.view,
            shape.content,
            shape.scrolled
        );
        if serial != self.measures.serial {
            return self;
        }
        Self {
            measures: Measures {
                cues: Some(Kept { list, cues, shape }),
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

/// Scrolls the cue list to `offset`
pub(super) fn scroll(offset: f32) -> Task<Message> {
    operation::scroll_to(
        view::CUES,
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
