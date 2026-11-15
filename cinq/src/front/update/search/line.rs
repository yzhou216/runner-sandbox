// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Editing the search line with the keys of Vim's command line: Control+w
//! and Control+u delete the word before the cursor and everything before
//! it, and ArrowUp and ArrowDown recall the older and newer searches that
//! start with the text typed
//!
//! iced does not tell where the cursor of the text field is, so the keys act
//! on the end of the text, where the cursor is while the operator types a
//! search.  Each key that changes the text moves the cursor to the end, as
//! Vim leaves it after a recall, so that the cursor and the keys agree.

use iced::{Task, widget::operation};

use super::{Search, Searched};
use crate::front::{App, Message, message::LineKey, view};

/// Most searches that the history keeps
const HISTORY: usize = 50;

/// Where the keys of recall are in the history, and the text typed before
/// the first of them, which they match against and which the keys of a
/// newer search give back
#[derive(Debug, Clone)]
pub(super) struct Recall {
    typed: String,
    /// Place in the history of the search recalled
    at: usize,
}

impl Searched {
    /// Returns the search accepted last with `text` and its direction, and
    /// `text` the newest in the history, once, without the oldest past the
    /// fiftieth
    pub(in crate::front::update) fn accepting(self, text: String, backward: bool) -> Self {
        let history: Vec<String> = self
            .history
            .into_iter()
            .filter(|old| *old != text)
            .chain([text.clone()])
            .collect();
        let past = history.len().saturating_sub(HISTORY);
        Self {
            text,
            backward,
            history: history.into_iter().skip(past).collect(),
        }
    }
}

impl App {
    /// Applies a key of the command line to the text of the search line,
    /// which selects the cue that the new text matches, as typing does
    pub(in crate::front::update) fn search_edited(self, key: LineKey) -> (Self, Task<Message>) {
        let Some(search) = &self.search else {
            return (self, Task::none());
        };
        let edited = match key {
            LineKey::DeleteWord => Some((without_last_word(&search.text), None)),
            LineKey::DeleteAll => Some((String::new(), None)),
            LineKey::Older => self.recalled(search, true),
            LineKey::Newer => self.recalled(search, false),
        };
        let Some((text, recall)) = edited.filter(|(text, _)| *text != search.text) else {
            return (self, Task::none());
        };
        let (app, task) = self.search_typed(text);
        let search = app.search.map(|search| Search { recall, ..search });
        (
            Self { search, ..app },
            task.chain(operation::move_cursor_to_end(view::SEARCH)),
        )
    }

    /// Returns the text of the nearest older or newer search that starts
    /// with the text typed, with where it is in the history, or the text
    /// typed when there is no newer one
    fn recalled(&self, search: &Search, older: bool) -> Option<(String, Option<Recall>)> {
        let history = &self.searched.history;
        let Recall { typed, at } = search.recall.clone().unwrap_or(Recall {
            typed: search.text.clone(),
            at: history.len(),
        });
        let starts = |index: &usize| history[*index].starts_with(&typed);
        let found = if older {
            (0..at).rev().find(starts)
        } else {
            (at + 1..history.len()).find(starts)
        };
        match found {
            Some(at) => Some((history[at].clone(), Some(Recall { typed, at }))),
            None if !older && search.recall.is_some() => Some((typed, None)),
            None => None,
        }
    }
}

/// Returns the text without the word at its end and the spaces after it, as
/// Control+w deletes the word before the cursor: a run of letters, digits,
/// and underscores, or else a run of other characters that are no spaces
pub(in crate::front::update) fn without_last_word(text: &str) -> String {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let trimmed = text.trim_end();
    let kind = trimmed.chars().next_back().map(word);
    let keep = trimmed
        .char_indices()
        .rev()
        .take_while(|&(_, c)| Some(word(c)) == kind && !c.is_whitespace())
        .last()
        .map_or(trimmed.len(), |(index, _)| index);
    trimmed[..keep].to_owned()
}
