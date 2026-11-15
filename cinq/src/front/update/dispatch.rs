// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What surrounds the handling of a message: handing commands to the back
//! end, deciding what a message of any kind does to a command typed halfway,
//! and running blocking work off the interface thread

use std::thread;

use iced::{Task, futures::channel::oneshot};

use crate::{
    back::Command,
    front::{App, Message},
};

impl App {
    pub(super) fn with_task(
        self,
        task: impl FnOnce(&Self) -> Task<Message>,
    ) -> (Self, Task<Message>) {
        let task = task(&self);
        (self, task)
    }

    /// Hands a command to the back end, and brings the show as its file
    /// would hold it up to date after a command that can change the cues.
    /// The list on screen stays within the lists, whose number undo and
    /// bringing back unsaved changes can change.
    pub(super) fn send(self, command: Command) -> Self {
        let changes = command.edits() || matches!(command, Command::Undo | Command::Redo);
        let workspace = self.workspace.apply(command);
        let app = Self {
            list: self.list.min(workspace.lists().len().saturating_sub(1)),
            workspace,
            ..self
        };
        if changes { app.with_text() } else { app }
    }

    pub(super) fn send_all(self, commands: impl IntoIterator<Item = Command>) -> Self {
        commands.into_iter().fold(self, Self::send)
    }

    /// Anything the operator does other than typing drops a command typed
    /// halfway, so that a stray `d` cannot wait through a click and an edit
    /// to delete a cue.  A level typed so far gives way to the level of its
    /// cue as well, apart from a click, which can land in its own field.  A
    /// link that failed to open is forgotten once the help page that holds
    /// it closes.
    pub(super) fn next(self, message: Message) -> (Self, Task<Message>) {
        let passive = matches!(
            message,
            Message::Tick(_)
                | Message::Loaded(..)
                | Message::Recheck
                | Message::Rechecked(..)
                | Message::LinkOpened(_)
                | Message::AudioDevicesListed(_)
                | Message::AudioDeviceKept(_)
                | Message::Saved(_)
                | Message::SearchSettled
                | Message::Resized(..)
                | Message::WindowFocused(_)
                | Message::Landed { .. }
                | Message::Reached { .. }
        );
        let typing = passive || matches!(message, Message::Typed { .. });
        let clicked = matches!(message, Message::Clicked);
        let app = Self {
            keys: if typing { self.keys } else { Vec::new() },
            clipped: self.clipped.filter(|_| typing),
            typed_level: self.typed_level.filter(|_| passive || clicked),
            ..self
        };
        let shown = format!("{message:?}");
        let moved = matches!(message, Message::Landed { .. } | Message::Reached { .. });
        let (app, task) = app.handled(message);
        let app = Self {
            link_problem: app.link_problem.filter(|_| app.help),
            ..app
        };
        if !passive || moved {
            sandbox!(
                "update {} | list={} rows={} selected_row={:?} selected_id={:?} keys={:?} help={} menu={} new_kind={:?} search={:?} clipped={:?}",
                shown.chars().take(100).collect::<String>(),
                app.list + 1,
                app.current().cues.len(),
                app.selected_row().map(|row| row + 1),
                app.selected,
                app.keys,
                app.help,
                app.menu,
                app.new_kind,
                app.search
                    .as_ref()
                    .map(|search| (search.text.clone(), search.backward)),
                app.clipped.as_ref().map(|clipped| clipped.told.clone()),
            );
        }
        (app, task)
    }
}

/// Runs blocking `work` on a thread of its own once iced runs the task: off
/// the interface thread, where GO and Panic must never wait, and off the
/// workers of iced's executor, which keep the clock ticking.  The task gives
/// what the work returns, or nothing when the program closes first.
pub(super) fn on_own_thread<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> Task<T> {
    Task::future(async move {
        let (sender, receiver) = oneshot::channel();
        thread::spawn(move || {
            // Receiver has gone only when the program is closing
            let _ = sender.send(work());
        });
        receiver.await.ok()
    })
    .and_then(Task::done)
}
