// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Hint mode: every place that a click acts on bears a label of letters, and
//! typing a label acts as the click would
//!
//! The app keeps the labels and the letters typed, and draws the labels in one
//! layer over the window.  A letter that no label continues is ignored, so
//! that a slip never runs the letters after it as commands, while any key that
//! is no letter leaves the mode and does nothing else, apart from the keys of
//! GO and Panic, which act as everywhere and leave the mode, and Backspace,
//! which takes a letter back, or leaves the mode where none is typed, since
//! the key of the mode is a letter of the labels in the vi keymap.  A click
//! leaves it too, and so do a turn of the wheel, a resize of the window, a
//! start or an end of a run of the Active list, and a warning that comes or
//! goes, since the labels would no longer lie on their places.

mod labels;

use iced::{
    Point, Task,
    keyboard::{Key, Modifiers, key::Named},
};

use super::{App, keys::reached};
use crate::{
    back::CueId,
    front::{
        Message,
        keymap::Press,
        view::{self, Act, Site, hint},
    },
};
use labels::labels;

/// Labels that hint mode has given out, each with its place, and the letters
/// typed so far
#[derive(Debug, Default, Clone)]
pub struct Hints {
    places: Vec<(String, Site)>,
    typed: String,
}

/// What the sidebar holds above the places that hint mode labels, which
/// decides where those places lie
#[derive(Debug, PartialEq)]
pub struct Above {
    active: Vec<CueId>,
    warnings: usize,
}

impl Hints {
    /// Returns hint mode with `typed` already typed, over no places
    #[cfg(test)]
    pub fn after(typed: &str) -> Self {
        Self {
            typed: typed.to_owned(),
            ..Self::default()
        }
    }

    /// Returns the letters typed so far, which the status bar shows
    pub fn typed(&self) -> &str {
        &self.typed
    }

    /// Returns the letters still to type of each label that begins with
    /// those typed, with the point of the window that its box begins at, which
    /// leaves its place in view
    pub fn labels(&self) -> impl Iterator<Item = (&str, Point)> {
        let shown: Vec<_> = self
            .places
            .iter()
            .filter_map(|(label, place)| {
                Some((label.strip_prefix(self.typed.as_str())?, place.shown))
            })
            .collect();
        let corners = hint::corners(
            &shown
                .iter()
                .map(|(letters, place)| (*place, letters.chars().count()))
                .collect::<Vec<_>>(),
        );
        shown
            .into_iter()
            .zip(corners)
            .map(|((letters, _), at)| (letters, at))
    }

    fn begun(&self, typed: &str) -> bool {
        self.places
            .iter()
            .any(|(label, _)| label.starts_with(typed))
    }
}

impl App {
    /// Turns hint mode on, which waits for the places that the widgets found,
    /// or turns it off
    pub(super) fn hinting(self) -> (Self, Task<Message>) {
        if self.hints.is_some() {
            return (self.without_hints(), Task::none());
        }
        (
            Self {
                hints: Some(Hints::default()),
                ..self
            },
            hint::places().map(Message::Hinted),
        )
    }

    /// Gives the places that the widgets found a label each, unless hint mode
    /// went off while they were found
    pub(super) fn hinted(self, places: Vec<Site>) -> Self {
        if self.hints.is_none() || places.is_empty() {
            return self.without_hints();
        }
        Self {
            hints: Some(Hints {
                places: labels(places.len()).into_iter().zip(places).collect(),
                typed: String::new(),
            }),
            ..self
        }
    }

    pub(super) fn without_hints(self) -> Self {
        Self {
            hints: None,
            ..self
        }
    }

    /// Returns what the sidebar holds above the places that hint mode labels:
    /// the cues of the Active list, in its order, and the number of warnings,
    /// which move those places when a run starts or ends or a warning comes
    /// or goes
    pub(super) fn above_places(&self) -> Above {
        Above {
            active: self
                .workspace
                .active()
                .iter()
                .map(|active| active.id)
                .collect(),
            warnings: view::warnings(self),
        }
    }

    /// Turns hint mode off unless the sidebar still holds what it held
    /// `before`, which is nothing to compare when the mode was off, since the
    /// places below an entry that came or went lie elsewhere than the labels
    /// do
    pub(super) fn hints_after(self, before: Option<&Above>) -> Self {
        match before {
            Some(before) if self.hints.is_some() && self.above_places() != *before => {
                self.without_hints()
            }
            _ => self,
        }
    }

    /// Reads a key that arrives while hint mode is on
    pub(super) fn hint_typed(self, press: Press, repeat: bool) -> (Self, Task<Message>) {
        let backspace = Press::from_key(&Key::Named(Named::Backspace), Modifiers::empty());
        let typed = self.hints.as_ref().map_or("", Hints::typed).to_owned();
        match self.config.keymap.alone(press) {
            Some(action) if action.acts_at_once() => self.without_hints().typed(press, repeat),
            // Held letter would pick the labels that it begins in a row
            _ if repeat => (self, Task::none()),
            _ if Some(press) == backspace => match typed.char_indices().next_back() {
                Some((last, _)) => self.hint_typed_as(typed[..last].to_owned()),
                None => (self.without_hints(), Task::none()),
            },
            _ => match press.letter() {
                // Labels are lowercase, whatever Caps Lock makes of the key
                Some(letter) => self.hint_typed_as(format!("{typed}{}", letter.to_lowercase())),
                None => (self.without_hints(), Task::none()),
            },
        }
    }

    /// Acts on the place whose label is `typed`, or goes on with those that
    /// it begins, or stays as it is if it begins none
    fn hint_typed_as(self, typed: String) -> (Self, Task<Message>) {
        let hints = self.hints.clone().unwrap_or_default();
        let whole = hints.places.iter().find(|(label, _)| *label == typed);
        match whole {
            Some((_, place)) => self.hint_chosen(place.clone()),
            None if hints.begun(&typed) => (
                Self {
                    hints: Some(Hints { typed, ..hints }),
                    ..self
                },
                Task::none(),
            ),
            None => (self, Task::none()),
        }
    }

    /// Acts as the click on `place` would
    fn hint_chosen(self, place: Site) -> (Self, Task<Message>) {
        let app = self.without_hints();
        match place.act {
            Act::Send(message) => app.next(message),
            Act::Reach => (app.without_search(), reached(hint::reach(place.at))),
        }
    }
}
