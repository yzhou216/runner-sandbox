// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Columns of the cue list, with the text of each

pub(super) mod states;

use itertools::Itertools;
use strum::{IntoStaticStr, VariantArray};
use unicode_width::UnicodeWidthStr;

use super::{RADIO, STATE_ROOM, TARGET_MOST};
use crate::{
    back::{Cue, Groups, Workspace, one_line},
    front::view::{char_width, format, widest},
};
pub(super) use states::{Look, counted, shown_lines};

#[derive(Clone, Copy, PartialEq, Eq, VariantArray, IntoStaticStr)]
pub(super) enum Column {
    #[strum(serialize = "#")]
    Number,
    Type,
    Name,
    Target,
    Pre,
    Action,
    Post,
    Then,
    State,
}

impl Column {
    /// Columns that give way in turn, the first to go first, when the list
    /// is too narrow to leave a name room to be read.  The Action column goes
    /// last, so that the State column can keep a word of its states and the
    /// name its least beside the number.
    pub(super) const YIELDING: [Column; 6] = [
        Column::Post,
        Column::Pre,
        Column::Then,
        Column::Type,
        Column::Target,
        Column::Action,
    ];

    pub(super) fn title(self) -> &'static str {
        self.into()
    }

    pub(super) fn text(self, workspace: &Workspace, groups: &Groups, cue: &Cue) -> String {
        match self {
            Column::Number => cue.number.clone(),
            Column::Type => cue.kind.to_string(),
            Column::Name => one_line(&cue.name),
            Column::Target => format::target(workspace, groups, cue),
            Column::Pre => format::wait(cue.pre_wait.value().into()),
            Column::Action => format::action(groups, cue),
            // Cue in a group goes on to no cue and ignores its post-wait
            Column::Post | Column::Then if groups.group_of(cue.id).is_some() => String::new(),
            Column::Post => format::post_wait(cue),
            Column::Then => format::advance(cue.advance).to_owned(),
            Column::State => {
                // Column has a fixed width, so that no output failing changes it
                let look = Look::default().with_next(groups.list().standby == Some(cue.id));
                shown_lines(workspace, groups, cue, look)
                    .into_iter()
                    .map(|(line, _)| line)
                    .join("\n")
            }
        }
    }

    /// Returns whether the list shows the column at all, which a yielding
    /// column that holds no text in any row does not, as it tells nothing.
    /// A list with no cue keeps them all, so that its header names them.
    pub(super) fn shown(self, workspace: &Workspace, groups: &Groups) -> bool {
        let cues = &groups.list().cues;
        !Self::YIELDING.contains(&self)
            || cues.is_empty()
            || cues
                .iter()
                .any(|cue| !self.text(workspace, groups, cue).is_empty())
    }

    /// Returns the columns of the font that the longest text of the column
    /// takes, its title included
    pub(super) fn widest(self, workspace: &Workspace, groups: &Groups) -> usize {
        groups
            .list()
            .cues
            .iter()
            .map(|cue| widest(self.text(workspace, groups, cue).lines()))
            .fold(self.title().width(), usize::max)
    }

    /// Returns none for the name, which takes the room left
    pub(super) fn width(self, workspace: &Workspace, groups: &Groups) -> Option<f32> {
        let widest = || self.widest(workspace, groups);
        let width = |columns: usize| columns as f32 * char_width();
        match self {
            Column::Name => None,
            Column::Number => Some(width(widest()) + RADIO),
            Column::Target => Some(width(widest().min(TARGET_MOST))),
            Column::State => Some(width(widest().max(STATE_ROOM))),
            _ => Some(width(widest())),
        }
    }
}
