// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Searching the rows in the main window, the cues of the list on screen or
//! the outputs, by the text of their columns, as Vim searches text
//!
//! A search finds the cues that a collapsed group hides, as Vim's finds text
//! in a closed fold, and selecting one expands the groups that hide it.  The
//! groups that the preview opens close again once the match moves elsewhere
//! or the search is cancelled, while accepting the search keeps them open.
//!
//! The search line opens at the left of the status bar, and the selection
//! follows the text typed, as with Vim's `incsearch`.  A search looks from
//! the row after the selection, or the row before it going up, and ends on
//! the selection itself.  Enter accepts the search, and Escape or Backspace
//! on an empty line gives the selection back to where it began.  The keys of
//! the next match repeat the search accepted last in its direction, and
//! those of the previous match in the other, wrapping around at either end
//! of the list, as `wrapscan` does.  The jumps of a search are the one that
//! Enter accepts, from where the search began to the match, and each step to
//! the next or previous match.  The moves made while typing and a cancel are
//! no jumps.  Every move that a search makes goes through
//! [`App::selecting_position`].

mod line;
mod matching;
mod previewing;
mod state;
mod stepping;

use iced::{Task, widget::operation};

use super::{focus::unfocus, keys::MAIN_WINDOW};
use crate::front::{App, Message, view};
use matching::{found, wrapped};
pub(in crate::front) use state::{Search, Searched};

/// What the status bar tells when a search goes past the end of the list and
/// goes on from the other end, as Vim tells when the search hits the bottom
fn continued(backward: bool) -> &'static str {
    if backward {
        "Search continued at the bottom"
    } else {
        "Search continued at the top"
    }
}

impl App {
    /// Opens the search line, and gives it the keyboard.  No field takes
    /// the keyboard while the stage window has it, since the keys typed
    /// there would never reach the field, so that the status bar says that
    /// the main window takes the key.
    pub(in crate::front) fn searching(self, backward: bool) -> (Self, Task<Message>) {
        if self.stage.has_keyboard() {
            return (
                Self {
                    notices: self.notices.with_remark(Some(MAIN_WINDOW)),
                    ..self
                },
                Task::none(),
            );
        }
        let search = Search::opened_at(self.spot(), backward);
        (
            Self {
                search: Some(search),
                ..self
            },
            operation::focus(view::SEARCH),
        )
    }

    /// Takes the text that the operator typed into the line, and selects
    /// the first row after where the search began that matches it, or else
    /// the row where the search began.  Backspace on a line with no text
    /// closes it.
    pub(super) fn search_typed(self, text: String) -> (Self, Task<Message>) {
        let Some(search) = &self.search else {
            return (self, Task::none());
        };
        if text.is_empty() && search.text.is_empty() {
            return self.search_cancelled().revealed_unfocused();
        }
        let (began, backward) = (search.began, search.backward);
        let row = self.position_of(began);
        let matched = found(&self.row_texts(), &text, row, backward);
        let missed = !text.is_empty() && matched.is_none();
        let went_round = wrapped(row, matched, backward);
        let app = Self {
            search: self.search.map(|search| Search {
                text,
                missed,
                warned: missed,
                recall: None,
                ..search
            }),
            notices: self
                .notices
                .with_remark(went_round.then(|| continued(backward))),
            ..self
        };
        app.search_reclosed()
            .selecting_position(matched.or(row))
            .search_previewed()
            .revealed()
    }

    /// Accepts the text of the line, and remembers it for the keys of the
    /// next and previous match.  A line with no text repeats the search
    /// accepted last, in the direction of the line, as Vim does, and one with
    /// text that no row matches stays open, so that it can tell so.
    pub(super) fn search_accepted(self) -> (Self, Task<Message>) {
        let Some(search) = &self.search else {
            return (self, Task::none());
        };
        if search.text.is_empty() {
            let searched = Searched {
                backward: search.backward,
                ..self.searched.clone()
            };
            let (app, task) = Self { searched, ..self }
                .search_cancelled()
                .search_stepped(1);
            return (app, Task::batch([task, unfocus()]));
        }
        if search.missed {
            return (self.warned(true), Task::none());
        }
        let began = search.began;
        let searched = self
            .searched
            .clone()
            .accepting(search.text.clone(), search.backward);
        let went_round = wrapped(self.position_of(began), self.position(), searched.backward);
        let remark = went_round.then(|| continued(searched.backward));
        // Moves of the preview were no jumps, so the jump is the one from
        // where the search began to the match
        Self {
            search: None,
            searched,
            notices: self.notices.with_remark(remark),
            ..self
        }
        .jumped_from(began)
        .revealed_unfocused()
    }

    fn warned(self, warned: bool) -> Self {
        Self {
            search: self.search.map(|search| Search { warned, ..search }),
            ..self
        }
    }

    /// Closes the line, if it is open, and puts the selection back where
    /// the search began
    pub(super) fn search_cancelled(self) -> Self {
        match self.search.as_ref().map(|search| search.began) {
            Some(began) => {
                let row = self.position_of(began);
                Self {
                    search: None,
                    ..self.search_reclosed().selecting_position(row)
                }
            }
            None => self,
        }
    }

    /// Closes the line as [`App::search_cancelled`] does, if it is open, and
    /// brings the selection that it put back into view, which leaves the view
    /// alone where no line was open
    pub(super) fn search_abandoned(self) -> (Self, Task<Message>) {
        if self.search.is_some() {
            self.search_cancelled().revealed()
        } else {
            (self, Task::none())
        }
    }

    /// Closes the line and keeps the selection where it is
    pub(super) fn without_search(self) -> Self {
        Self {
            search: None,
            ..self
        }
    }

    pub(super) fn search_settled(self) -> Self {
        self.warned(false)
    }

    fn revealed_unfocused(self) -> (Self, Task<Message>) {
        let (app, reveal) = self.revealed();
        (app, Task::batch([reveal, unfocus()]))
    }
}
