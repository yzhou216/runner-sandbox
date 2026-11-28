// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Opening a show, which puts it on screen along with the unsaved changes
//! kept for it

use std::path::Path;

use iced::Task;

use super::{Read, Show, chooser, named, open};
use crate::{
    back::{Command, Workspace},
    front::{App, History, Message, Notices, Problems, keymap::Action, view::answered},
};

impl App {
    /// Takes in why the keeper failed to keep changes, if it did, which
    /// replaces an older reason.  A failure also takes back the note that
    /// changes were kept, which it contradicts.
    pub(in crate::front::update) fn with_keep_problem(self, failed: Option<String>) -> Self {
        Self {
            notices: Notices {
                told: self.notices.told.filter(|_| failed.is_none()),
                ..self.notices
            },
            problems: self.problems.with_keep(failed),
            ..self
        }
    }

    pub(in crate::front::update) fn choose_show(&self) -> Task<Message> {
        let chooser = chooser(self.show.directory()).set_title("Open show");
        let reader = self.reader.clone();
        Task::future(chooser.pick_file())
            .and_then(move |chosen| open(Some(chosen.path().to_owned()), reader.clone()))
    }

    /// The show that was on screen keeps its own unsaved changes, which the
    /// keeper keeps as they happen.  A show opened again while it is on
    /// screen with unsaved changes brings back the changes on screen, since
    /// its kept file may still wait for the newest of them.
    pub(in crate::front::update) fn opened(self, read: Read) -> Self {
        let name = read
            .file
            .as_deref()
            .map_or_else(|| "a new show".to_owned(), |file| named(file).into_owned());
        let workspace = match workspace_of(read.text, &read.directory) {
            Ok(workspace) => workspace,
            Err(problem) => {
                return Self {
                    problems: Problems {
                        file: Some(format!("Cannot open {name}: {problem}")),
                        open_failed: true,
                        ..self.problems
                    },
                    ..self
                };
            }
        };
        let saved = workspace.show(read.file.as_deref().and_then(Path::parent));
        let again = read.file == self.show.file;
        let kept = if self.show.unsaved() && again {
            Workspace::from_show(&self.show.text, &read.directory).map(Some)
        } else {
            read.kept.and_then(|kept| {
                kept.map(|text| Workspace::from_show(&text, &read.directory))
                    .transpose()
            })
        };
        // Show that goes keeps its unsaved changes, which opening it again
        // brings back, as starting cinq without a show does for a new show
        let told = (self.show.unsaved() && !again).then(|| self.kept_note());
        let opened = Self {
            workspace,
            show: Show {
                id: self.show.id + 1,
                file: read.file,
                saved: saved.clone(),
                text: saved,
                version: self.show.version + 1,
            },
            problems: self.problems.without_files(),
            // Labels stand for the places of the show that goes
            hints: None,
            ..self
        };
        // Selection and message once the kept changes are back, since
        // bringing them back replaces the cues and clears what was told
        opened.with_kept(kept, &name).starting(told)
    }

    fn kept_note(&self) -> String {
        let kept = format!("Kept the unsaved changes to {}", self.show.name());
        match self.show.file {
            Some(_) => answered(kept, &self.config.keymap, Action::Open),
            None => kept,
        }
    }

    /// Brings back the kept changes only when they differ from the show as
    /// saved, or notes why they cannot be read
    fn with_kept(self, kept: Result<Option<Workspace>, String>, name: &str) -> Self {
        match kept {
            Ok(Some(kept)) if kept.show(self.show.directory()) != self.show.saved => {
                self.send(Command::Restore(
                    kept.lists().to_vec(),
                    kept.outputs().to_vec(),
                    kept.patch().to_vec(),
                ))
            }
            Ok(_) => self,
            Err(problem) => Self {
                problems: Problems {
                    file: Some(format!(
                        "Cannot bring back the unsaved changes to {name}: {problem}"
                    )),
                    ..self.problems
                },
                ..self
            },
        }
    }

    /// Shows the first cue of the show just opened, with `told` as the
    /// message
    fn starting(self, told: Option<String>) -> Self {
        let opened = Self {
            notices: self.notices.with_told(told),
            // Cue and output identities restart from one in each show
            history: History::default(),
            scroll: None,
            output: None,
            ..self
        }
        .switched_to(0)
        .without_search();
        // Outputs that stay in view select the first of the new show
        if opened.outputs_shown {
            opened.showing_outputs()
        } else {
            opened
        }
    }
}

/// Reads the workspace of a show from its text, or an empty one when the
/// file does not exist yet
fn workspace_of(
    text: Result<Option<String>, String>,
    directory: &Path,
) -> Result<Workspace, String> {
    text.and_then(|text| match text {
        Some(text) => Workspace::from_show(&text, directory),
        None => Ok(Workspace::default().directory(directory)),
    })
}
