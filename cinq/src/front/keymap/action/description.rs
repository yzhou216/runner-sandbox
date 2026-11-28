// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sentence that the help page gives for each action

use super::Action;

impl Action {
    pub fn description(self) -> &'static str {
        match self {
            Action::Go => "GO: fire the standby cue and its chain",
            Action::Panic => "Panic: fade everything out, or stop it at once",
            Action::Standby => "Put the selected cue on standby",
            Action::Cancel => {
                "Drop the keys typed so far, close the search line or a menu, and leave a field or hint mode"
            }
            Action::Next => "Select the next cue, output, or menu option",
            Action::Previous => "Select the previous cue, output, or menu option",
            Action::First => "Select the first cue, output, or menu option, or the row counted",
            Action::Last => "Select the last cue, output, or menu option, or the row counted",
            Action::HalfPageDown => {
                "Scroll half a page down with the selection, or the rows counted"
            }
            Action::HalfPageUp => "Scroll half a page up with the selection, or the rows counted",
            Action::PageDown => {
                "Scroll a page down with the selection, or the pages counted, or move one in a menu"
            }
            Action::PageUp => {
                "Scroll a page up with the selection, or the pages counted, or move one in a menu"
            }
            Action::ScrollDown => "Scroll the list a row down, or the rows counted",
            Action::ScrollUp => "Scroll the list a row up, or the rows counted",
            Action::TopRow => "Select the first row in view, or the row counted from it",
            Action::MiddleRow => "Select the middle row in view",
            Action::BottomRow => "Select the last row in view, or the row counted up from it",
            Action::RowToMiddle => {
                "Scroll the list to put the selected row, or the row counted, in the middle"
            }
            Action::RowToTop => {
                "Scroll the list to put the selected row, or the row counted, at the top"
            }
            Action::RowToBottom => {
                "Scroll the list to put the selected row, or the row counted, at the bottom"
            }
            Action::Collapse => {
                "Collapse the selected group, or the group that holds the selected cue"
            }
            Action::Expand => "Expand the selected group",
            Action::ToggleGroup => {
                "Expand the selected group if it is collapsed, or else collapse it"
            }
            Action::CollapseAll => "Collapse every group of the list",
            Action::ExpandAll => "Expand every group of the list",
            Action::NextList => "Show the next cue list, then the outputs",
            Action::PreviousList => "Show the previous cue list, or the last from the outputs",
            Action::NextTab => "Show the next inspector tab, or the tab counted",
            Action::PreviousTab => {
                "Show the previous inspector tab, or go back as many tabs as counted"
            }
            Action::Back => "Go back to the place that the last jump left, or leave the outputs",
            Action::Forward => "Go forward again after going back",
            Action::Search => "Search the cues, or the outputs, by the text of their columns",
            Action::SearchBackward => {
                "Search the cues, or the outputs, by the text of their columns, upwards"
            }
            Action::NextMatch => "Select the next match of the search, in its direction",
            Action::PreviousMatch => {
                "Select the match that lies in the opposite direction from the last search"
            }
            Action::Edit => {
                "Edit the name of the selected cue, or output, with the caret at its end"
            }
            Action::EditStart => {
                "Edit the name of the selected cue, or output, with the caret at its start"
            }
            Action::ChangeName => {
                "Change the name of the selected cue, or output, selecting all of it"
            }
            Action::NextField => "Move to the next field",
            Action::PreviousField => "Move to the previous field",
            Action::Hint => {
                "Label the buttons, check boxes, tabs, drop-down lists, fields, and links, to act on one by typing its label"
            }
            Action::Pick => "Pick the highlighted option of an open menu",
            Action::CloseMenu => "Close an open menu without picking an option",
            Action::AddBelow => "Add a cue, or an output, below the selection",
            Action::AddAbove => "Add a cue, or an output, above the selection",
            Action::Cut => "Cut the selected cue, or output, to the clipboard",
            Action::Copy => "Copy the selected cue, or output, to the clipboard",
            Action::PasteBelow => "Paste the copied cues, or outputs, below the selection",
            Action::PasteAbove => "Paste the copied cues, or outputs, above the selection",
            Action::Delete => "Delete the selected cue, or output, without copying it",
            Action::MoveDown => "Move the selected cue, or output, down",
            Action::MoveUp => "Move the selected cue, or output, up",
            Action::Indent => "Move the selected cue into the group above it",
            Action::Outdent => "Move the selected cue out of its group",
            Action::Undo => "Undo the latest change to the show, or as many as counted",
            Action::Redo => "Redo the change undone last, or as many as counted",
            Action::Open => "Open a show",
            Action::Save => "Save the show",
            Action::SaveAs => "Save the show in another file",
            Action::Fullscreen => "Toggle fullscreen on the stage window",
            Action::Help => "Show or hide the help page",
            Action::CloseHelp => "Hide the help page",
        }
    }
}

/// Rows that the keys of cues act on, which are the cues of a cue list or
/// the outputs of the show, according to which the main window shows
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rows {
    Cues,
    Outputs,
}

/// Words of the help page that name both kinds of rows, with what each kind
/// reads as alone, the longest first so that a phrase is never cut short
const BOTH: [(&str, &str, &str); 8] = [
    ("the cues, or the outputs,", "the cues", "the outputs"),
    (
        "cue, or output, selecting",
        "cue, selecting",
        "output, selecting",
    ),
    (
        "cue, output, or menu option",
        "cue or menu option",
        "output or menu option",
    ),
    ("a cue, or an output,", "a cue", "an output"),
    ("cues, or outputs,", "cues", "outputs"),
    ("cue, or output,", "cue", "output"),
    ("cue or output", "cue", "output"),
    ("cues or outputs", "cues", "outputs"),
];

impl Rows {
    /// Returns what the rows are called, and what many of them are
    pub(super) fn nouns(self) -> (&'static str, &'static str) {
        match self {
            Rows::Cues => ("cue", "cues"),
            Rows::Outputs => ("output", "outputs"),
        }
    }
}

impl Action {
    /// Returns the sentence of [`Action::description_in`], except that an
    /// operator with a `count` typed reads as what its doubled key takes, the
    /// selected row and the next ones that make up the count
    pub fn description_counted(self, rows: Rows, count: Option<usize>) -> String {
        match count {
            // Doubled operator takes the rows counted, the selected one included
            Some(count) if count > 1 && Self::OPERATORS.contains(&self) => {
                self.over(Action::Next, rows, Some(count - 1))
            }
            _ => self.description_in(rows),
        }
    }

    /// Returns the sentence of the help page for the rows that show, so that
    /// the box of the keys that can follow names outputs where the outputs
    /// show and cues where a cue list does, instead of both
    pub(in crate::front::keymap) fn description_in(self, rows: Rows) -> String {
        BOTH.iter().fold(
            self.description().to_owned(),
            |text, &(both, cues, outputs)| {
                text.replace(both, if rows == Rows::Cues { cues } else { outputs })
            },
        )
    }
}
