// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sentence that the help page gives for each action

use super::Action;

impl Action {
    pub fn description(self) -> &'static str {
        match self {
            Action::Go => "GO: fire the standby cue and its chain",
            Action::Panic => "Panic: fade everything out, or stop it at once",
            Action::Standby => "Put the selected cue on standby",
            Action::Cancel => {
                "Drop the keys typed so far, close the search line or a menu, and leave a field"
            }
            Action::Next => "Select the next cue or menu option",
            Action::Previous => "Select the previous cue or menu option",
            Action::First => "Select the first cue or menu option, or the row counted",
            Action::Last => "Select the last cue or menu option, or the row counted",
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
            Action::ScrollDown => "Scroll the cue list a row down, or the rows counted",
            Action::ScrollUp => "Scroll the cue list a row up, or the rows counted",
            Action::TopRow => "Select the first row in view, or the row counted from it",
            Action::MiddleRow => "Select the middle row in view",
            Action::BottomRow => "Select the last row in view, or the row counted up from it",
            Action::RowToMiddle => {
                "Scroll the cue list to put the selected row, or the row counted, in the middle"
            }
            Action::RowToTop => {
                "Scroll the cue list to put the selected row, or the row counted, at the top"
            }
            Action::RowToBottom => {
                "Scroll the cue list to put the selected row, or the row counted, at the bottom"
            }
            Action::NextList => "Show the next cue list",
            Action::PreviousList => "Show the previous cue list",
            Action::NextTab => "Show the next inspector tab, or the tab counted",
            Action::PreviousTab => "Show the previous inspector tab, or go back as many as counted",
            Action::Back => "Go back to the place that the last jump left",
            Action::Forward => "Go forward again after going back",
            Action::Search => "Search the cues by name",
            Action::SearchBackward => "Search the cues by name, upwards",
            Action::NextMatch => "Select the next match of the search, in its direction",
            Action::PreviousMatch => {
                "Select the next match of the search, in the opposite direction"
            }
            Action::Edit => "Edit the name of the selected cue, with the caret at its end",
            Action::EditStart => "Edit the name of the selected cue, with the caret at its start",
            Action::ChangeName => "Change the name of the selected cue, selecting all of it",
            Action::NextField => "Move to the next field",
            Action::PreviousField => "Move to the previous field",
            Action::Pick => "Pick the highlighted option of an open menu",
            Action::CloseMenu => "Close an open menu without picking an option",
            Action::AddBelow => "Add a cue below the selection",
            Action::AddAbove => "Add a cue above the selection",
            Action::Cut => "Cut the selected cue to the clipboard",
            Action::Copy => "Copy the selected cue to the clipboard",
            Action::PasteBelow => "Paste the copied cues below the selection",
            Action::PasteAbove => "Paste the copied cues above the selection",
            Action::Delete => "Delete the selected cue without copying it",
            Action::MoveDown => "Move the selected cue down",
            Action::MoveUp => "Move the selected cue up",
            Action::Undo => "Undo the latest change to the show, or as many as counted",
            Action::Redo => "Redo the change undone last, or as many as counted",
            Action::Open => "Open a show",
            Action::Save => "Save the show",
            Action::SaveAs => "Save the show in another file",
            Action::Fullscreen => "Toggle fullscreen on the stage window",
            Action::Help => "Show or hide this help",
            Action::CloseHelp => "Hide this help",
        }
    }
}
