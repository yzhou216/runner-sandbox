// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Columns of the cue list: the text of each cell, the width of each
//! column, and the state of a cue, in the color of what it needs

use iced::{Theme, widget::text};
use strum::{IntoStaticStr, VariantArray};
use unicode_width::UnicodeWidthStr;

use super::{RADIO, STATE_ROOM, TARGET_ROOM};
use crate::{
    back::{Cue, CueList, Problem, Workspace},
    front::view::{char_width, format},
};

/// Column of the cue list, which goes by its name, apart from the number
#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantArray, IntoStaticStr)]
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
    /// Columns that give way in turn when the list is too narrow to leave a
    /// name room to be read, after those that are empty in every row
    pub(super) const YIELDING: [Column; 5] = [
        Column::Post,
        Column::Pre,
        Column::Then,
        Column::Type,
        Column::Target,
    ];

    pub(super) fn title(self) -> &'static str {
        self.into()
    }

    /// Returns the text of the cell of `cue` in this column
    pub(super) fn text(self, workspace: &Workspace, list: &CueList, cue: &Cue) -> String {
        match self {
            Column::Number => cue.number.clone(),
            Column::Type => cue.kind.to_string(),
            Column::Name => cue.name.clone(),
            Column::Target => format::target(workspace, cue),
            Column::Pre => format::wait(cue.pre_wait.value().into()),
            Column::Action => format::action(cue),
            Column::Post => format::post_wait(cue),
            Column::Then => format::advance(cue.advance).to_owned(),
            Column::State => state(workspace, list, cue).0,
        }
    }

    /// Returns whether the column has text in any row of `list`
    pub(super) fn holds_text(self, workspace: &Workspace, list: &CueList) -> bool {
        list.cues
            .iter()
            .any(|cue| !self.text(workspace, list, cue).is_empty())
    }

    /// Returns the width of the column, as wide as its title or its longest
    /// cell, or none for the name, which takes the room left
    pub(super) fn width(self, workspace: &Workspace, list: &CueList) -> Option<f32> {
        let widest = || {
            list.cues
                .iter()
                .map(|cue| self.text(workspace, list, cue).width())
                .fold(self.title().width(), usize::max)
        };
        let width = |columns: usize| columns as f32 * char_width();
        match self {
            Column::Name => None,
            Column::Number => Some(width(widest()) + RADIO),
            Column::Target => Some(width(widest().min(TARGET_ROOM))),
            Column::State => Some(width(STATE_ROOM)),
            _ => Some(width(widest())),
        }
    }
}

/// Describes a cue in the State column, in the color of what it needs: red
/// while it cannot play, yellow while it is paused, green while it runs,
/// and blue while it waits on standby for GO.  A cue that cannot play names
/// what it lacks, a file or a target, or else is BROKEN, as when its file
/// cannot be played or its start is not before its end.  A file that is
/// still loading stops the cue from playing too, but it needs nothing but
/// time.  The list of running cues tells more of each, such as where a
/// paused cue stopped.
pub(super) fn state(
    workspace: &Workspace,
    list: &CueList,
    cue: &Cue,
) -> (String, fn(&Theme) -> text::Style) {
    match (workspace.problem(cue), workspace.active_of(cue.id)) {
        (Some(Problem::NotLoaded), _) => ("loading".to_owned(), text::default),
        (Some(Problem::NoFile), _) => ("NO FILE".to_owned(), text::danger),
        (Some(Problem::NoTarget), _) => ("NO TARGET".to_owned(), text::danger),
        (Some(Problem::LoadFailed(_) | Problem::EmptyClip), _) => {
            ("BROKEN".to_owned(), text::danger)
        }
        (None, Some(active)) if active.paused => ("paused".to_owned(), text::warning),
        (None, Some(active)) => (format::phase(active), text::success),
        (None, None) if list.standby == Some(cue.id) => ("NEXT".to_owned(), text::primary),
        (None, None) => (String::new(), text::default),
    }
}
