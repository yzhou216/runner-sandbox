// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Messages that send a command to the show, or take a setting typed

use super::super::{App, Message};
use crate::back::{Command, OutputEdit, PatchEdit};

impl App {
    /// Applies a message that sends a command to the show, or takes the text
    /// of a setting, and passes any other on to [`Self::paneled_by`]
    pub(super) fn sent_by(self, message: Message) -> Self {
        match message {
            Message::Copy(count) => self.with_copied(count),
            Message::PauseAll => self.send(Command::Pause(true)),
            Message::ResumeAll => self.send(Command::Pause(false)),
            Message::Stop(id) => self.send(Command::Stop(id)),
            // Nothing without a selected cue, as the toolbar's button does
            Message::StandbySelected => {
                let standby = self.selected_here().map(|cue| Command::Standby {
                    list: self.list,
                    cue: Some(cue),
                });
                self.send_all(standby)
            }
            Message::ShowMode(show_mode) => self.send(Command::ShowMode(show_mode)),
            Message::SettingTyped(setting, text) => {
                self.settled_but(setting).typed_setting(setting, text)
            }
            Message::OutputDevice(output, device) => {
                self.send(Command::EditOutput(output, OutputEdit::Device(device)))
            }
            Message::Edit(id, edit) => self.send(Command::Edit(id, edit)),
            Message::Patch(PatchEdit::Add) => {
                let held = self.workspace.patch().len();
                let app = self.send(Command::Patch(PatchEdit::Add));
                // The last instrument holds the last address of the last universe
                let remark = (app.workspace.patch().len() == held)
                    .then_some("The light patch has no address left for another instrument");
                Self {
                    notices: app.notices.with_remark(remark),
                    ..app
                }
            }
            Message::Patch(edit) => self.send(Command::Patch(edit)),
            message => self.paneled_by(message),
        }
    }
}
