// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Loading the files of audio, video, and picture cues, and looking again
//! at those that have been read, for a file that has changed or gone, or
//! that has come back
//!
//! Reading a file can block for a long time when its drive is slow or has
//! gone, so the files are read on a thread of their own.  One load and one
//! recheck run at a time, so that a read that hangs piles up no reads
//! behind it, and a recheck that hangs never holds back a file just named.
//! A recheck reads only the files whose stamps have changed, as
//! [`load_files`] tells, and the subscription wakes it only when
//! [`watch`](crate::back::watch) finds a file that differs from what
//! its cue read.

use iced::Task;

use super::on_own_thread;
use crate::{
    back::{Command, Load, load_files},
    front::{App, Message},
};

impl App {
    pub(super) fn load(self) -> (Self, Task<Message>) {
        let loads = Load::all(&self.workspace, self.workspace.unloaded());
        if self.loading || loads.is_empty() {
            return (self, Task::none());
        }
        Self {
            loading: true,
            ..self
        }
        .with_task(|app| read(loads, app.show.id(), Message::Loaded))
    }

    pub(super) fn loaded(self, show: u64, found: Vec<Command>) -> Self {
        Self {
            loading: false,
            ..self.with_found(show, found)
        }
    }

    pub(super) fn recheck(self) -> (Self, Task<Message>) {
        let loads = Load::all(&self.workspace, self.workspace.to_recheck());
        if self.rechecking || loads.is_empty() {
            return (self, Task::none());
        }
        Self {
            rechecking: true,
            ..self
        }
        .with_task(|app| read(loads, app.show.id(), Message::Rechecked))
    }

    pub(super) fn rechecked(self, show: u64, found: Vec<Command>) -> Self {
        Self {
            rechecking: false,
            ..self.with_found(show, found)
        }
    }

    /// Records what reading the files of the show of identity `show`
    /// found, unless another show has taken its place since
    fn with_found(self, show: u64, found: Vec<Command>) -> Self {
        if show == self.show.id() {
            self.send_all(found)
        } else {
            self
        }
    }
}

fn read(loads: Vec<Load>, show: u64, done: fn(u64, Vec<Command>) -> Message) -> Task<Message> {
    on_own_thread(move || load_files(loads)).map(move |found| done(show, found))
}
