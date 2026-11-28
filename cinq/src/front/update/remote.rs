// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Commands that arrive over OSC, which do what the keys of GO and Panic and
//! the buttons of the transport do

use std::net::IpAddr;

use iced::Task;

use super::{App, focus::unfocus};
use crate::{
    back::{Command, CueId, Problem, Remote},
    front::Message,
    warn,
};

/// Most characters of a cue number that a warning holds, since anyone on the
/// network can send it
const SHOWN: usize = 64;

impl App {
    /// Carries out a command from the network.  It leaves hint mode as a
    /// click does, and acts in show mode, while a field has the keyboard,
    /// and over the help page, as GO and Panic do, since none of the
    /// commands edits the show.  A command that moves the selection or the
    /// list in view takes the keyboard out of a text field and closes the
    /// search line, as going back does, so that what is typed next cannot
    /// edit another cue.  A cue number that no cue has, and a start of a cue that cannot play, do nothing to the
    /// show, and to the keys typed halfway, and tell where they came from and
    /// what they named, on standard error once and in the sidebar until a
    /// later command acts.
    pub(super) fn remote(self, from: IpAddr, remote: Remote) -> (Self, Task<Message>) {
        if let Some(reason) = self.ignored(&remote) {
            let line = format!("OSC from {from}: {reason}");
            warn::once(&line);
            return (self.with_ignored(line), Task::none());
        }
        let here = (self.selected, self.list);
        let (app, task) = Self {
            problems: self.problems.with_osc_ignored(None),
            ..self
        }
        .carried_out(remote);
        if (app.selected, app.list) == here {
            (app, task)
        } else {
            (app.without_search(), Task::batch([task, unfocus()]))
        }
    }

    /// Returns the state with `line` as the command that did nothing.  A first
    /// line moves what lies below it in the sidebar, so that it leaves hint mode.
    pub(super) fn with_ignored(self, line: String) -> Self {
        let app = if self.problems.osc_ignored.is_none() {
            self.without_hints()
        } else {
            self
        };
        Self {
            problems: app.problems.with_osc_ignored(Some(line)),
            ..app
        }
    }

    /// Returns why `remote` changes nothing at all, the interface included:
    /// no cue has the number that it aims at, or it starts a cue that cannot
    /// play.  A cue that only waits for its file to load is no problem, as
    /// it plays once the file is there.
    pub(super) fn ignored(&self, remote: &Remote) -> Option<String> {
        let number = remote.number()?;
        match self.workspace.numbered(number) {
            None => {
                let shown: String = number.chars().take(SHOWN).collect();
                Some(format!("no cue is numbered {shown:?}"))
            }
            Some(cue) => matches!(remote, Remote::Start(_))
                .then(|| self.workspace.problem(cue))
                .flatten()
                .filter(|problem| *problem != Problem::NotLoaded)
                .map(|problem| format!("cue {number} cannot play: {problem}")),
        }
    }

    fn carried_out(self, remote: Remote) -> (Self, Task<Message>) {
        let app = self.without_hints();
        match remote {
            Remote::Go => app.handled(Message::Go),
            Remote::Panic => app.handled(Message::Panic),
            Remote::Pause => (app.send(Command::Pause(true)), Task::none()),
            Remote::Resume => (app.send(Command::Pause(false)), Task::none()),
            Remote::Start(number) => app.with_numbered(&number, Self::started),
            Remote::Stop(number) => app.with_numbered(&number, |app, id| {
                (app.send(Command::Stop(id)), Task::none())
            }),
            Remote::PauseCue(number) => app.with_numbered(&number, |app, id| {
                (app.send(Command::Hold(id)), Task::none())
            }),
            Remote::ResumeCue(number) => app.with_numbered(&number, |app, id| {
                (app.send(Command::Resume(id)), Task::none())
            }),
            Remote::Playhead(number) => {
                app.with_numbered(&number, |app, id| app.send(Command::GoTo(id)).picked(id))
            }
        }
    }

    fn with_numbered(
        self,
        number: &str,
        act: impl FnOnce(Self, CueId) -> (Self, Task<Message>),
    ) -> (Self, Task<Message>) {
        match self.workspace.numbered(number).map(|cue| cue.id) {
            Some(id) => act(self, id),
            // Unreachable, since [`Self::remote`] looked the number up
            None => (self, Task::none()),
        }
    }

    /// Fires a cue as a start cue does, and cues the stage window as GO does
    /// when a cue that it fires shows on stage
    fn started(self, id: CueId) -> (Self, Task<Message>) {
        // A paused cue only resumes, which fires nothing
        let resuming = self.workspace.active_of(id).is_some_and(|run| run.paused);
        let staged = !resuming
            && self
                .workspace
                .firing(id)
                .iter()
                .any(|(cue, _)| cue.kind.on_stage());
        let app = self.send(Command::Start(id));
        let stage = if staged { app.stage.cued() } else { app.stage };
        (Self { stage, ..app }, Task::none())
    }
}
