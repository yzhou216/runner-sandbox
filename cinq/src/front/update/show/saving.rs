// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Saving the show in its file, or in another that the operator chooses

use std::{
    io,
    path::{Path, PathBuf},
};

use iced::Task;

use super::{EXTENSION, Saved, Show, chooser, file::write, named, with_extension};
use crate::{
    front::{App, Message, Problems, update::on_own_thread},
    warn,
};

/// Writes the whole `error` of saving `file` on standard error, with the
/// paths that the screen leaves out, and returns its words for the screen,
/// as `permission denied`.  The temporary file that tempfile creates hides
/// the system's error behind its path, so that only the kind goes on.
pub(super) fn refused(file: &Path, error: &io::Error) -> String {
    warn::once(&format!("cannot save {}: {error}", file.display()));
    warn::words(&io::Error::from(error.kind()))
}

impl App {
    pub(in crate::front::update) fn choose_file(&self) -> Task<Message> {
        let chooser = chooser(self.show.directory())
            .set_title("Save show")
            .set_file_name(match &self.show.file {
                Some(_) => self.show.name().into_owned(),
                None => format!("untitled.{EXTENSION}"),
            });
        Task::future(chooser.save_file())
            .and_then(|chosen| Task::done(Message::SaveTo(with_extension(chosen.path()))))
    }

    pub(in crate::front::update) fn save(&self) -> Task<Message> {
        match self.show.file.clone() {
            Some(file) => self.save_to(file),
            None => self.choose_file(),
        }
    }

    /// Relative paths of the files of the cues go from the directory of
    /// `file`
    pub(in crate::front::update) fn save_to(&self, file: PathBuf) -> Task<Message> {
        let show = self.show.id;
        let text = self.workspace.show(file.parent());
        on_own_thread(move || Saved {
            result: write(&file, &text, true).map_err(|error| refused(&file, &error)),
            show,
            file,
            text,
        })
        .map(Message::Saved)
    }

    /// The show then goes by the file it went to, and any change made while
    /// it was being saved stays unsaved.  A save that ends after another show
    /// has opened leaves that show alone.
    pub(in crate::front::update) fn saved(self, saved: Saved) -> Self {
        match saved.result {
            Ok(()) if saved.show != self.show.id => self,
            Ok(()) => {
                let show = Show {
                    text: self.workspace.show(saved.file.parent()),
                    file: Some(saved.file),
                    saved: saved.text,
                    version: self.show.version + 1,
                    ..self.show
                };
                Self {
                    notices: self
                        .notices
                        .with_told(Some(format!("Saved {}", show.name()))),
                    problems: self.problems.without_files(),
                    show,
                    ..self
                }
            }
            Err(reason) => Self {
                // Name of the show already stands at the left of the bar
                notices: self
                    .notices
                    .with_told(Some(format!("Cannot save: {reason}"))),
                problems: Problems {
                    file: Some(format!("Cannot save {}: {reason}", named(&saved.file))),
                    open_failed: false,
                    ..self.problems
                },
                ..self
            },
        }
    }
}
