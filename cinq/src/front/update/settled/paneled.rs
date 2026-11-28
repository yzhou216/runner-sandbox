// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Messages that change which panel the sidebar shows

use super::super::{App, Message};
use crate::front::Tab;

impl App {
    /// Applies a message that shows the help page, a tab, or an output, and
    /// leaves the state as it is for any other
    pub(super) fn paneled_by(self, message: &Message) -> Self {
        match message {
            Message::SelectOutput(output) => Self {
                output: Some(*output),
                ..self
            },
            Message::ToggleHelp => Self {
                help: !self.help,
                ..self
            },
            Message::CloseHelp => Self {
                help: false,
                ..self
            },
            Message::Tab(tab) => Self { tab: *tab, ..self },
            Message::ShowTab(number) => Self {
                tab: Tab::numbered(*number),
                ..self
            },
            Message::StepTab(offset) => Self {
                tab: self.tab.stepped(*offset),
                ..self
            },
            // Guarded arms above that do not match, and messages that need
            // no state change, leave the state as it is
            _ => self,
        }
    }
}
