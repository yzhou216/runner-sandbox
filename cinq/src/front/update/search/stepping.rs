// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The keys of the next and previous match, which repeat the search that
//! Enter accepted last

use iced::Task;

use super::{
    Searched, continued,
    matching::{found, matching, within_a_lap, wrapped},
};
use crate::front::{App, Message, view::no_match};

impl App {
    /// Selects the row `offset` matches of the last search further on in the
    /// direction that it went, or the other way when negative, wrapping
    /// around.  It does nothing before a search, or when no row matches.  Vim
    /// counts the keys of the next and previous match among its jumps, so
    /// this one is a jump.
    pub(in crate::front::update) fn search_stepped(self, offset: isize) -> (Self, Task<Message>) {
        let Searched { text, backward, .. } = &self.searched;
        let backward = *backward != (offset < 0);
        let texts = self.row_texts();
        let found_rows = matching(&texts, text);
        let steps = within_a_lap(offset.unsigned_abs(), found_rows);
        let (row, went_round) = (0..steps).fold((self.position(), false), |(row, round), _| {
            let next = found(&texts, text, row, backward);
            (next.or(row), round || wrapped(row, next, backward))
        });
        let remark = if text.is_empty() {
            // Before any search was accepted, as Vim tells of no previous regular
            // expression
            Some("No previous search")
        } else if found_rows == 0 {
            Some(no_match(self.outputs_shown))
        } else {
            went_round.then(|| continued(backward))
        };
        Self {
            notices: self.notices.with_remark(remark),
            ..self
        }
        .jumping(|app| app.selecting_position(row))
        .revealed()
    }
}
