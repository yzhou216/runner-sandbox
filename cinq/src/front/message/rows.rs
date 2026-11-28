// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Messages that act on the rows of the list in the main window

use super::Message;

impl Message {
    /// Returns whether the message moves the selection through the rows of
    /// the list in the main window, scrolls it, or searches it.  The cue list
    /// and the outputs share these.
    pub fn moves_through_rows(&self) -> bool {
        matches!(
            self,
            Message::Step(_)
                | Message::Row(_)
                | Message::Page(..)
                | Message::Landed { .. }
                | Message::Scroll(_)
                | Message::InView(..)
                | Message::Reached { .. }
                | Message::Place(..)
                | Message::Search { .. }
                | Message::Searching(_)
                | Message::SearchKey(_)
                | Message::SearchAccepted
                | Message::SearchStep(_)
        )
    }

    /// Returns whether the message acts on the rows of the cue list: those
    /// that [`Message::moves_through_rows`] names, and those that act on a
    /// cue, edit rows, or change the inspector tab.  While the outputs show
    /// in place of the cue list, the ones that are not shared must not
    /// reach it.
    pub fn acts_on_rows(&self) -> bool {
        self.moves_through_rows()
            || matches!(
                self,
                Message::StandbySelected
                    | Message::StepTab(_)
                    | Message::ShowTab(_)
                    | Message::Add(..)
                    | Message::Move(_)
                    | Message::Indent(_)
                    | Message::Outdent(_)
                    | Message::Fold(_)
                    | Message::FlipGroup(_)
                    | Message::Copy(_)
                    | Message::Cut(_)
                    | Message::Paste(..)
                    | Message::Delete(_)
                    | Message::Across(..)
                    | Message::Undo(_)
                    | Message::Redo(_)
            )
    }
}
