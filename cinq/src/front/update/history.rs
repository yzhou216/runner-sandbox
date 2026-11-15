// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The places that jumps left, which going back and forward walk along, as
//! the jump list of Vim does
//!
//! A place is a cue list with a selected cue, or a cue list with none, or the
//! outputs with a selected output, or with none.  The history lives in the interface state only, so that neither the show file
//! nor undo ever holds it.  A jump after going back moves the places ahead
//! behind, in the order of the list of Vim, so that none is lost.

mod places;

use iced::{
    Task,
    advanced::widget::{operate, operation::focusable},
};

use crate::front::{App, Message};
use places::alive;
pub(in crate::front) use places::{History, Spot};

impl App {
    pub(super) fn spot(&self) -> Spot {
        if self.outputs_shown {
            self.output.map_or(Spot::Outputs, Spot::Output)
        } else {
            self.selected_here()
                .map_or(Spot::List(self.list), Spot::Cue)
        }
    }

    fn at(self, spot: Spot) -> Self {
        match spot {
            Spot::Cue(id) => self.showing_cues().with_selection(id),
            Spot::List(list) => Self {
                list,
                selected: None,
                ..self.showing_cues()
            },
            Spot::Output(output) => Self {
                output: Some(output),
                ..self
            }
            .showing_outputs(),
            Spot::Outputs => self.showing_outputs(),
        }
    }

    /// Runs `go`, a command that moves to another place by what it is, and
    /// remembers the place that it left unless it stayed.  The place is read
    /// before `go` runs, so that a `go` which shows the cue list in place of
    /// the outputs still remembers where it came from.
    pub(super) fn jumping(self, go: impl FnOnce(Self) -> Self) -> Self {
        let from = self.spot();
        go(self).jumped_from(from)
    }

    /// Remembers `from` as the place that a jump left, unless the jump
    /// stayed there
    pub(super) fn jumped_from(self, from: Spot) -> Self {
        if self.spot() == from {
            self
        } else {
            Self {
                history: self.history.jumped(from),
                ..self
            }
        }
    }

    /// Runs `go` as [`App::jumping`] does, but remembers the place only when
    /// the view changed, to another cue list or to the cue list in place of
    /// the outputs, since a click on a cue of the list on screen points at
    /// what is in view
    pub(super) fn crossing(self, go: impl FnOnce(Self) -> Self) -> Self {
        let (from, view) = (self.spot(), (self.outputs_shown, self.list));
        let app = go(self);
        if (app.outputs_shown, app.list) == view {
            app
        } else {
            Self {
                history: app.history.jumped(from),
                ..app
            }
        }
    }

    /// Runs `walk`, a move along the history, and takes the keyboard out of
    /// a text field when the place changed, so that what is typed next
    /// cannot edit the cue arrived at.  An open search line closes, and the
    /// selection stays where the search put it.
    pub(super) fn walking(self, walk: impl FnOnce(Self) -> Self) -> (Self, Task<Message>) {
        let from = self.spot();
        let (app, revealing) = walk(self.without_search()).revealed();
        let leaving = if app.spot() == from {
            Task::none()
        } else {
            operate(focusable::unfocus())
        };
        (app, Task::batch([revealing, leaving]))
    }

    /// Goes back along the places that jumps left.  The outputs view has
    /// always a way out, even where no jump recorded one, as after opening a
    /// show: the cue list that was showing, with the cue that was selected.
    pub(super) fn went_back(self, count: usize) -> Self {
        let here = self.spot();
        let back = self
            .history
            .back(here, count, |spot| alive(&self.workspace, spot));
        match back {
            Some((spot, history)) => Self { history, ..self }.at(spot),
            None if self.outputs_shown => {
                let app = self.showing_cues();
                Self {
                    history: app.history.left(here),
                    ..app
                }
            }
            None => self,
        }
    }

    pub(super) fn went_forward(self, count: usize) -> Self {
        let here = self.spot();
        let forward = self
            .history
            .forward(here, count, |spot| alive(&self.workspace, spot));
        match forward {
            Some((spot, history)) => Self { history, ..self }.at(spot),
            None => self,
        }
    }

    pub(in crate::front) fn can_go_back(&self) -> bool {
        self.outputs_shown
            || self
                .history
                .can_go_back(self.spot(), |spot| alive(&self.workspace, spot))
    }

    pub(in crate::front) fn can_go_forward(&self) -> bool {
        self.history
            .can_go_forward(self.spot(), |spot| alive(&self.workspace, spot))
    }
}
