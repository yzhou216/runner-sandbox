// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cutting, copying, deleting, indenting, and outdenting the rows that a
//! motion reaches, as `d`, `y`, `>`, and `<` do in Vim
//!
//! The motions are the actions that move the selection, and the rows are
//! whole rows of the cue list, or of the outputs while they show, from the
//! selection to where the motion lands, inclusive.

use std::ops::RangeInclusive;

use crate::{
    back::Command,
    front::{App, keymap::Action},
};

impl App {
    /// Applies `operator` to the rows that `motion` reaches and leaves the
    /// selection on the first of them, or on the row that takes its place
    /// when the operator takes the rows away.  Nothing happens without a
    /// selection, where the motion cannot move, or, for the operators that
    /// change the show, in show mode, which leaves copying alone as it does
    /// elsewhere.  Outputs lie in no group, so that indenting and
    /// outdenting do nothing while they show.
    pub(in crate::front::update) fn across(
        self,
        operator: Action,
        motion: Action,
        count: Option<usize>,
    ) -> Self {
        let Some(span) = self.reached(operator, motion, count) else {
            return self;
        };
        let rows = span.end() - span.start() + 1;
        let app = self.at_row(span.start() + 1);
        match (app.outputs_shown, operator) {
            (false, Action::Copy) => app.with_copied(rows),
            (false, Action::Delete) => app.without_selected(rows),
            (false, Action::Indent) => app.regrouped(rows, Command::Indent),
            (false, Action::Outdent) => app.regrouped(rows, Command::Outdent),
            (false, _) => app.cut(rows),
            (true, Action::Copy) => app.copied_outputs(rows),
            (true, Action::Delete) => app.without_outputs(rows),
            (true, _) => app.cut_outputs(rows),
        }
    }
}

impl App {
    /// Returns the rows that `operator` acts on over `motion`, counted from
    /// zero, or nothing where [`App::across`] changes nothing
    pub(in crate::front::update) fn reached(
        &self,
        operator: Action,
        motion: Action,
        count: Option<usize>,
    ) -> Option<RangeInclusive<usize>> {
        let grouping = matches!(operator, Action::Indent | Action::Outdent);
        self.row()
            .filter(|_| self.editable() || operator == Action::Copy)
            .filter(|_| !(grouping && self.outputs_shown))
            .and_then(|row| rows(motion, count, row, self.row_count()))
    }
}

/// Returns the rows from `row` to where `motion` lands, counted from zero,
/// or nothing where it cannot move, as in Vim.  A count past the last row
/// stops at it.  For `First` and `Last`, the count is the row, counted from
/// one, that they land on.
fn rows(
    motion: Action,
    count: Option<usize>,
    row: usize,
    len: usize,
) -> Option<RangeInclusive<usize>> {
    let last = len.checked_sub(1)?;
    let times = count.unwrap_or(1);
    let landing = match motion {
        Action::Next if row < last => row.saturating_add(times).min(last),
        Action::Previous if row > 0 => row.saturating_sub(times),
        Action::First => times.saturating_sub(1).min(last),
        Action::Last => count.map_or(last, |count| count.saturating_sub(1).min(last)),
        _ => return None,
    };
    Some(row.min(landing)..=row.max(landing))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_and_previous_move_by_the_count_and_stop_at_the_ends_or_do_nothing_there() {
        assert_eq!(rows(Action::Next, None, 3, 10), Some(3..=4));
        assert_eq!(rows(Action::Next, Some(4), 3, 10), Some(3..=7));
        assert_eq!(rows(Action::Next, Some(99), 3, 10), Some(3..=9));
        assert_eq!(rows(Action::Previous, None, 3, 10), Some(2..=3));
        assert_eq!(rows(Action::Previous, Some(99), 3, 10), Some(0..=3));
        // Nothing happens where the selection cannot move
        assert_eq!(rows(Action::Next, None, 9, 10), None);
        assert_eq!(rows(Action::Previous, Some(3), 0, 10), None);
    }

    #[test]
    fn first_and_last_reach_the_row_counted_in_either_direction() {
        assert_eq!(rows(Action::Last, None, 3, 10), Some(3..=9));
        assert_eq!(rows(Action::First, None, 3, 10), Some(0..=3));
        assert_eq!(rows(Action::First, Some(2), 5, 10), Some(1..=5));
        assert_eq!(rows(Action::First, Some(8), 5, 10), Some(5..=7));
        assert_eq!(rows(Action::Last, Some(8), 5, 10), Some(5..=7));
        assert_eq!(rows(Action::Last, Some(99), 5, 10), Some(5..=9));
        // Rows that the selection is on
        assert_eq!(rows(Action::Last, None, 9, 10), Some(9..=9));
        assert_eq!(rows(Action::First, None, 0, 10), Some(0..=0));
    }

    #[test]
    fn no_cues_and_other_actions_reach_no_rows() {
        assert_eq!(rows(Action::Next, None, 0, 0), None);
        assert_eq!(rows(Action::Undo, None, 3, 10), None);
    }
}
