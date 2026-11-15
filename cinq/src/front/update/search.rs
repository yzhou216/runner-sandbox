// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Searching the cues of the list on screen by name and number, as Vim
//! searches text
//!
//! The search line opens at the left of the status bar, and the selection
//! follows the text typed, as with Vim's `incsearch`.  A search looks from
//! the row after the selection, or the row before it going up, and ends on
//! the selection itself, as Vim's does from the start of a line.  Enter
//! accepts the search, and Escape or Backspace on an empty line gives the
//! selection back to where it began.  The keys of the next match repeat the
//! search accepted last in its direction, and those of the previous match in
//! the other, and wrap around at either end of the list, as `wrapscan` does
//! in Vim.  The jumps of a search, which Vim lists among its
//! jump motions, are the one that Enter accepts, from where the search began
//! to the match, and each step to the next or previous match.  The moves
//! made while typing and a cancel are no jumps.  Every move that a search
//! makes goes through [`App::searched_to`].

mod matching;

use iced::{
    Task,
    advanced::widget::{operate, operation::focusable},
    widget::operation,
};

use super::history::Spot;
use crate::{
    back::CueId,
    front::{App, Message, view},
};
use matching::{found, matching, within_a_lap};

/// Search line while it is open
pub(in crate::front) struct Search {
    pub text: String,
    /// Whether the search goes up the list, as Vim's `?` does
    pub backward: bool,
    /// Selection as it was when the line opened, where cancelling the search
    /// puts it back and where the search looks first
    began: Spot,
    /// Whether no cue matches the text
    missed: bool,
    /// Whether the line tells so, which it does for a while
    pub warned: bool,
}

/// Search accepted last, which the keys of the next and previous match
/// repeat
#[derive(Default)]
pub(in crate::front) struct Searched {
    pub text: String,
    pub backward: bool,
}

impl Search {
    fn opened_at(began: Spot, backward: bool) -> Self {
        Self {
            text: String::new(),
            backward,
            began,
            missed: false,
            warned: false,
        }
    }
}

impl App {
    /// Opens the search line, and gives it the keyboard.  No field takes
    /// the keyboard while the stage window has it, since the keys typed
    /// there would never reach the field.
    pub(super) fn searching(self, backward: bool) -> (Self, Task<Message>) {
        if self.stage.has_keyboard() {
            return (self, Task::none());
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

    /// Moves the selection to the cue of a search, which is the one place
    /// where a search changes it.  It records nothing: the callers that
    /// make a jump remember the place that it left.
    fn searched_to(self, cue: Option<CueId>) -> Self {
        Self {
            selected: cue,
            ..self
        }
    }

    /// Takes the text that the operator typed into the line, and selects
    /// the first cue after where the search began that matches it, or else
    /// the cue where the search began.  Backspace on a line with no text
    /// closes it.
    pub(super) fn search_typed(self, text: String) -> (Self, Task<Message>) {
        let Some(search) = &self.search else {
            return (self, Task::none());
        };
        if text.is_empty() && search.text.is_empty() {
            return self.search_cancelled().revealed_unfocused();
        }
        let (began, backward) = (search.began, search.backward);
        let row = began.cue().and_then(|began| self.current().position(began));
        let cue = found(&self.current().cues, &text, row, backward)
            .map(|row| self.current().cues[row].id);
        let missed = !text.is_empty() && cue.is_none();
        let app = Self {
            search: Some(Search {
                text,
                backward,
                missed,
                warned: missed,
                began,
            }),
            ..self
        };
        app.searched_to(cue.or(began.cue())).revealed()
    }

    /// Accepts the text of the line, and remembers it for the keys of the
    /// next and previous match.  A line with no text repeats the search
    /// accepted last, in the direction of the line, as Vim does, and one with
    /// text that no cue matches stays open, so that it can tell so.
    pub(super) fn search_accepted(self) -> (Self, Task<Message>) {
        let Some(search) = &self.search else {
            return (self, Task::none());
        };
        if search.text.is_empty() {
            let searched = Searched {
                text: self.searched.text.clone(),
                backward: search.backward,
            };
            let (app, task) = Self { searched, ..self }
                .search_cancelled()
                .search_stepped(1);
            return (app, Task::batch([task, operate(focusable::unfocus())]));
        }
        if search.missed {
            return (self.search_missed(), Task::none());
        }
        let (began, searched) = (
            search.began,
            Searched {
                text: search.text.clone(),
                backward: search.backward,
            },
        );
        // The moves of the preview were no jumps, so the jump is the one from
        // where the search began to the match
        Self {
            search: None,
            searched,
            ..self
        }
        .jumped_from(began)
        .revealed_unfocused()
    }

    fn search_missed(self) -> Self {
        Self {
            search: self.search.map(|search| Search {
                warned: true,
                ..search
            }),
            ..self
        }
    }

    /// Closes the line, if it is open, and puts the selection back where
    /// the search began
    pub(super) fn search_cancelled(self) -> Self {
        match self.search.as_ref().map(|search| search.began) {
            Some(began) => Self {
                search: None,
                ..self.searched_to(began.cue())
            },
            None => self,
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
        Self {
            search: self.search.map(|search| Search {
                warned: false,
                ..search
            }),
            ..self
        }
    }

    /// Selects the cue `offset` matches of the last search further on in the
    /// direction that it went, or the other way when negative, wrapping
    /// around.  It does nothing before a search, or when no cue matches.  Vim
    /// counts the keys of the next and previous match among its jumps, so
    /// this one is a jump.
    pub(super) fn search_stepped(self, offset: isize) -> (Self, Task<Message>) {
        let Searched { text, backward } = &self.searched;
        let backward = *backward != (offset < 0);
        let cues = &self.current().cues;
        let steps = within_a_lap(offset.unsigned_abs(), matching(cues, text));
        let row = (0..steps).fold(self.selected_row(), |row, _| {
            found(cues, text, row, backward).or(row)
        });
        let cue = row.map(|row| cues[row].id).or(self.selected);
        self.jumping(|app| app.searched_to(cue)).revealed()
    }

    fn revealed_unfocused(self) -> (Self, Task<Message>) {
        let (app, reveal) = self.revealed();
        (app, Task::batch([reveal, operate(focusable::unfocus())]))
    }
}
