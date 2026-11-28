// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Entry of a message into the update, [`App::next`], and what surrounds the
//! handling of it: deciding what a message of any kind does to a command
//! typed halfway, and handing commands to the back end

use iced::Task;

use super::{editing::Stopping, focus::reveal_current_list};
use crate::{
    back::Command,
    front::{App, Message, Notices, app::Held},
};

/// Returns whether `message` reports something that happened without the
/// operator, which changes nothing that the operator was in the middle of,
/// and moves no keyboard but for a window taking it, which drops the keys
/// typed halfway
pub(super) fn passive(message: &Message) -> bool {
    matches!(
        message,
        Message::Tick(_)
            | Message::Loaded(..)
            | Message::Recheck
            | Message::Rechecked(..)
            | Message::NetworkChecked
            | Message::LightTold(_)
            | Message::OscFailed(_)
            | Message::OscIgnored(_)
            | Message::OscListening
            | Message::LinkOpened(_)
            | Message::AudioDevicesListed(..)
            | Message::AudioDeviceKept(_)
            | Message::Saved(_)
            | Message::SearchSettled
            | Message::Resized(..)
            | Message::WindowFocused(_)
            | Message::Held(_)
            | Message::Landed { .. }
            | Message::Measured(_)
            | Message::Remeasure
            | Message::Hinted(_)
            | Message::Drawn(_)
            | Message::Reached { .. }
    )
}

impl App {
    /// Returns whether `message` is a command that changes nothing, as one
    /// for a cue number that no cue has does, which interrupts nothing that
    /// the operator was doing, as a passive message does
    fn aimless(&self, message: &Message) -> bool {
        matches!(message, Message::Remote(_, remote) if self.ignored(remote).is_some())
    }

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
        let changes = command.edits()
            || matches!(
                command,
                Command::Undo | Command::Redo | Command::Collapse { .. }
            );
        let turns =
            self.notices.turns + usize::from(matches!(command, Command::Undo | Command::Redo));
        let stopping = Stopping::of(&command, &self.workspace);
        let workspace = self.workspace.apply(command);
        let app = Self {
            list: self.list.min(workspace.lists().len().saturating_sub(1)),
            notices: Notices {
                stopped: stopping.stopped(&self.notices.stopped, &workspace),
                turns,
                ..self.notices
            },
            workspace,
            ..self
        };
        if changes { app.with_text() } else { app }
    }

    pub(super) fn send_all(self, commands: impl IntoIterator<Item = Command>) -> Self {
        commands.into_iter().fold(self, Self::send)
    }

    /// Applies a message, after anything other than typing drops what the
    /// operator left half done.  A command typed halfway goes, so that a stray
    /// `d` cannot wait through a click and an edit to delete a cue.  A level
    /// or the channels typed into a field apply once it is left or at any
    /// click, before the message acts, so that GO fires a cue at the level
    /// typed and no key after a click opens a device.  A link that failed to
    /// open is forgotten once its help page closes.
    pub(super) fn next(self, message: Message) -> (Self, Task<Message>) {
        let passive = passive(&message) || self.aimless(&message);
        let typing = passive || matches!(message, Message::Typed { .. });
        let clicked = matches!(message, Message::Clicked);
        // Typing into a field keeps its text, which the operator leaves with
        // anything else, Enter included
        let in_field = matches!(message, Message::SettingTyped(..));
        let kept = passive || clicked || in_field;
        // Field that the keyboard enters or leaves ends the run of edits to its
        // text, so that each visit makes a change of its own to undo, as each
        // stay in insert mode does in Vim
        let ends = matches!(message, Message::Focus(_) | Message::Clicked)
            || (matches!(message, Message::Held(Held { any: false, .. }))
                && self.keyboard.held.any);
        let this = if ends {
            self.send(Command::EndRun)
        } else {
            self
        };
        // Field that the keyboard leaves takes its rejected text with it, so
        // that the status bar tells the reason, which the field no longer does
        let unapplied = (!kept).then(|| this.unapplied()).flatten();
        let app = if passive || in_field {
            this
        } else {
            this.settled()
        };
        // A key sequence waits through nothing that moves the keyboard to
        // another window, as one that another program took and gave back
        let waiting = typing && !matches!(message, Message::WindowFocused(_));
        let app = Self {
            keys: if waiting { app.keys } else { Vec::new() },
            notices: app.notices.while_typing(typing),
            typed: app.typed.filter(|_| kept),
            ..app
        };
        let shown = app.shown_list();
        let (app, task) = app.handled(message);
        let task = if app.shown_list() == shown {
            task
        } else {
            task.chain(reveal_current_list())
        };
        let app = Self {
            problems: app.problems.with_link_while(app.help),
            notices: app.notices.preferring_unapplied(unapplied),
            ..app
        };
        (app, task)
    }
}
