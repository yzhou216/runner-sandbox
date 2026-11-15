// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The message that an action sends, with the count typed before its keys

use super::Action;
use crate::{
    back::Place,
    front::{
        Caret, Focus, Message,
        message::{Height, Page},
    },
};

impl Action {
    pub fn message(self, count: Option<usize>) -> Message {
        let times = count.unwrap_or(1);
        let step = isize::try_from(times).unwrap_or(isize::MAX);
        match self {
            Action::Go => Message::Go,
            Action::Panic => Message::Panic,
            Action::Standby => Message::StandbySelected,
            Action::Next => Message::Step(step),
            Action::Previous => Message::Step(-step),
            Action::First => Message::Row(times),
            Action::Last => Message::Row(count.unwrap_or(usize::MAX)),
            Action::HalfPageDown => Message::Page(Page::HalfDown, count),
            Action::HalfPageUp => Message::Page(Page::HalfUp, count),
            Action::PageDown => Message::Page(Page::WholeDown, count),
            Action::PageUp => Message::Page(Page::WholeUp, count),
            Action::ScrollDown => Message::Scroll(step),
            Action::ScrollUp => Message::Scroll(-step),
            Action::TopRow => Message::InView(Height::Top, count),
            Action::MiddleRow => Message::InView(Height::Middle, count),
            Action::BottomRow => Message::InView(Height::Bottom, count),
            Action::RowToMiddle => Message::Place(Height::Middle),
            Action::RowToTop => Message::Place(Height::Top),
            Action::RowToBottom => Message::Place(Height::Bottom),
            Action::NextList => Message::StepList(step),
            Action::PreviousList => Message::StepList(-step),
            Action::NextTab => count.map_or(Message::StepTab(1), Message::ShowTab),
            Action::PreviousTab => Message::StepTab(-step),
            Action::Back => Message::Back(times),
            Action::Forward => Message::Forward(times),
            Action::Search => Message::Search { backward: false },
            Action::SearchBackward => Message::Search { backward: true },
            Action::NextMatch => Message::SearchStep(step),
            Action::PreviousMatch => Message::SearchStep(-step),
            Action::Edit => Message::Focus(Focus::Name(Caret::End)),
            Action::EditStart => Message::Focus(Focus::Name(Caret::Start)),
            Action::ChangeName => Message::Focus(Focus::Name(Caret::All)),
            Action::NextField => Message::Focus(Focus::Next),
            Action::PreviousField => Message::Focus(Focus::Previous),
            Action::Pick | Action::CloseMenu => Message::Nothing,
            Action::AddBelow => Message::Add(Place::After, times),
            Action::AddAbove => Message::Add(Place::Before, times),
            Action::Cut => Message::Cut(times),
            Action::Copy => Message::Copy(times),
            Action::PasteBelow => Message::Paste(Place::After, times),
            Action::PasteAbove => Message::Paste(Place::Before, times),
            Action::Delete => Message::Delete(times),
            Action::MoveDown => Message::Move(step),
            Action::MoveUp => Message::Move(-step),
            Action::Undo => Message::Undo(times),
            Action::Redo => Message::Redo(times),
            Action::Open => Message::Open,
            Action::Save => Message::Save,
            Action::SaveAs => Message::SaveAs,
            Action::Fullscreen => Message::ToggleFullscreen,
            Action::Help => Message::ToggleHelp,
            Action::CloseHelp => Message::CloseHelp,
        }
    }
}
