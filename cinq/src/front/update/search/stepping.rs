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
use crate::front::{App, Message, view::NO_MATCH};

impl App {
    /// Selects the cue `offset` matches of the last search further on in the
    /// direction that it went, or the other way when negative, wrapping
    /// around.  It does nothing before a search, or when no cue matches.  Vim
    /// counts the keys of the next and previous match among its jumps, so
    /// this one is a jump.
    pub(in crate::front::update) fn search_stepped(self, offset: isize) -> (Self, Task<Message>) {
        let Searched { text, backward, .. } = &self.searched;
        let backward = *backward != (offset < 0);
        let cues = &self.current().cues;
        let found_cues = matching(cues, text);
        let steps = within_a_lap(offset.unsigned_abs(), found_cues);
        let (row, went_round) = (0..steps).fold((self.selected_row(), false), |(row, round), _| {
            let next = found(cues, text, row, backward);
            let round = round
                || row
                    .zip(next)
                    .is_some_and(|(from, to)| wrapped(from, to, backward));
            (next.or(row), round)
        });
        let cue = row.map(|row| cues[row].id).or(self.selected);
        let remark = if found_cues == 0 {
            Some(NO_MATCH)
        } else {
            went_round.then(|| continued(backward))
        };
        Self { remark, ..self }
            .jumping(|app| app.searched_to(cue))
            .revealed()
    }
}
