// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The rows that the main window lists, which the keys that move through
//! rows share
//!
//! The main window lists the cues of a cue list or, in place of them, the
//! outputs of the show.  Selecting, stepping, paging, scrolling, and placing a
//! row in view go through the interface here, so that a key does the same to
//! either list by one code path.  Editing differs by kind and lives with
//! each kind.
//!
//! A row of the cues is a row that the table shows, so a collapsed group is
//! one row, with every cue that it holds.  A search looks through the cues
//! themselves, as Vim searches the text of a closed fold, and finds the
//! cues that a collapsed group hides by their position in the list instead.

mod folding;
mod history;
mod navigation;
pub(super) mod paging;
pub(super) mod scroll;
pub(super) mod viewing;

use crate::front::{App, Spot, stepping::step, view};

pub(in crate::front) use history::History;
pub(in crate::front) use paging::kept::{Measured, Measures};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rows {
    /// Cues of the cue list with this index
    Cues(usize),
    Outputs,
}

impl App {
    pub(super) fn rows(&self) -> Rows {
        if self.outputs_shown {
            Rows::Outputs
        } else {
            Rows::Cues(self.list)
        }
    }

    pub(super) fn row_count(&self) -> usize {
        match self.rows() {
            Rows::Cues(_) => self.current().groups().shown().len(),
            Rows::Outputs => self.workspace.outputs().len(),
        }
    }

    /// Returns the selected row, counted from zero
    pub(super) fn row(&self) -> Option<usize> {
        match self.rows() {
            Rows::Cues(_) => self.selected_row(),
            Rows::Outputs => self.output_row(),
        }
    }

    fn scrollable(&self) -> &'static str {
        match self.rows() {
            Rows::Cues(_) => view::CUES,
            Rows::Outputs => view::OUTPUTS,
        }
    }

    /// Selects the item on a row, counted from zero, or no item for none,
    /// leaving the selection of the list that is out of sight alone
    pub(super) fn selecting(self, row: Option<usize>) -> Self {
        match self.rows() {
            Rows::Cues(_) => {
                let list = self.current();
                let selected = row
                    .and_then(|row| list.groups().shown().get(row).copied())
                    .map(|index| list.cues[index].id);
                Self { selected, ..self }
            }
            Rows::Outputs => {
                let output = row
                    .and_then(|row| self.workspace.outputs().get(row))
                    .map(|output| output.id);
                Self { output, ..self }
            }
        }
    }

    /// Returns the places in the cue list of the cues that `count` rows from
    /// the row `first` down show, a collapsed group standing for itself
    pub(super) fn shown_cues_from(&self, first: usize, count: usize) -> Vec<usize> {
        self.current()
            .groups()
            .shown()
            .into_iter()
            .skip(first)
            .take(count)
            .collect()
    }

    /// Returns where a search stands, which is the place in the cue list of
    /// the selected cue, hidden by a collapsed group or not, or the row of
    /// the selected output
    pub(super) fn position(&self) -> Option<usize> {
        match self.rows() {
            Rows::Cues(_) => self.selected_index(),
            Rows::Outputs => self.output_row(),
        }
    }

    /// Returns where a search stands for the place that a spot names, if it
    /// is in the items listed
    pub(super) fn position_of(&self, spot: Spot) -> Option<usize> {
        match (self.rows(), spot) {
            (Rows::Cues(_), Spot::Cue(id)) => self.current().position(id),
            (Rows::Outputs, Spot::Output(id)) => self
                .workspace
                .outputs()
                .iter()
                .position(|output| output.id == id),
            _ => None,
        }
    }

    /// Selects the item at a position of the search texts, the cue in the
    /// cue list or the row of the outputs.  A cue that a collapsed group
    /// hides is selected as any other, and the groups expand once the
    /// selection is brought into view.
    pub(super) fn selecting_position(self, position: Option<usize>) -> Self {
        match self.rows() {
            Rows::Cues(_) => {
                let selected = position
                    .and_then(|index| self.current().cues.get(index))
                    .map(|cue| cue.id);
                Self { selected, ..self }
            }
            Rows::Outputs => self.selecting(position),
        }
    }

    /// Returns the texts of each item that a search matches, a cue of the
    /// list whether the table shows it or not: those that the row shows in
    /// its columns, but for the state, as Vim searches a whole line
    pub(super) fn row_texts(&self) -> Vec<Vec<String>> {
        match self.rows() {
            Rows::Cues(_) => {
                let groups = self.current().groups();
                groups
                    .list()
                    .cues
                    .iter()
                    .map(|cue| view::cue_texts(&self.workspace, &groups, cue))
                    .collect()
            }
            Rows::Outputs => self
                .workspace
                .outputs()
                .iter()
                .map(|output| view::output_texts(self, output))
                .collect(),
        }
    }

    pub(super) fn stepped(self, offset: isize) -> Self {
        let row = step(self.row(), offset, self.row_count());
        self.selecting(row)
    }

    /// Selects the row counted from one, or the last row for a row past the
    /// end
    pub(super) fn at_row(self, row: usize) -> Self {
        let last = self.row_count().checked_sub(1);
        let row = last.map(|last| row.saturating_sub(1).min(last));
        self.selecting(row)
    }
}
