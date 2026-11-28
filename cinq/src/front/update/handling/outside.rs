// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Messages that reach the files, the links, and the audio devices of the
//! computer

use iced::Task;

use super::{
    super::{device, edit_config, open_link, open_tutorial},
    App, Message,
};

impl App {
    pub(super) fn outside_by(self, message: Message) -> (Self, Task<Message>) {
        match message {
            Message::Recheck => self.recheck(),
            Message::Open => self.with_task(Self::choose_show),
            Message::Opened(read) => self.opened(*read).revealed(),
            Message::Save => self.with_task(Self::save),
            Message::SaveAs => self.with_task(Self::choose_file),
            Message::SaveTo(file) => self.with_task(|app| app.save_to(file)),
            Message::ChooseFile(id) => self.with_task(|app| app.choose_cue_file(id)),
            Message::OpenLink(target) => self.opening(move || open_link(&target)),
            Message::EditConfig(file) => self.opening(move || edit_config(&file)),
            Message::OpenTutorial => self.opening(open_tutorial),
            Message::ListAudioDevices => self.list_devices(),
            Message::ChooseAudioDevice(device) => self.chose_audio_device(device),
            message => (self.settled_by(message), Task::none()),
        }
    }
}
