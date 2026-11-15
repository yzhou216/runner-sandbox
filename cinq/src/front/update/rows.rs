// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The rows that the main window lists, which the keys that move through
//! rows share
//!
//! The main window lists the cues of a cue list or, in place of them, the
//! outputs of the show.  Selecting, stepping, paging, scrolling, placing a
//! row in view, and searching go through the interface here, so that a key
//! does the same to either list by one code path.  Editing differs by kind
//! and lives with each kind.

use crate::front::{App, step, view};

use super::history::Spot;

/// Rows that the main window lists
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
            Rows::Cues(_) => self.current().cues.len(),
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

    /// Returns the identity of the scrollable that holds the rows
    pub(super) fn scrollable(&self) -> &'static str {
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
                let selected = row
                    .and_then(|row| self.current().cues.get(row))
                    .map(|cue| cue.id);
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

    /// Returns the row that a place names, if it is one of the rows listed
    pub(super) fn row_of(&self, spot: Spot) -> Option<usize> {
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

    /// Returns the texts of each row that a search matches: the name and the
    /// number of a cue, or the name of an output
    pub(super) fn row_texts(&self) -> Vec<Vec<&str>> {
        match self.rows() {
            Rows::Cues(_) => self
                .current()
                .cues
                .iter()
                .map(|cue| vec![cue.name.as_str(), cue.number.as_str()])
                .collect(),
            Rows::Outputs => self
                .workspace
                .outputs()
                .iter()
                .map(|output| vec![output.name.as_str()])
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
