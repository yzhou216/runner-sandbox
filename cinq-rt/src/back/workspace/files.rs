// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the files of cues are, which relative file names find from the
//! directory of the show

use std::path::{Path, PathBuf};

use super::Workspace;
use crate::back::Cue;

impl Workspace {
    /// Returns the workspace with relative file paths starting from
    /// `directory`, the way QLab finds files next to its workspace file.
    /// Without one, they start from the current directory.
    pub fn directory(self, directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
            ..self
        }
    }

    pub(in crate::back) fn path(&self, cue: &Cue) -> Option<PathBuf> {
        cue.file_name().map(|name| self.directory.join(name))
    }

    /// Returns the directory to look in for a file for `cue` to play: the
    /// directory of the file it plays now, or else the one that relative
    /// paths start from
    pub fn file_directory(&self, cue: &Cue) -> PathBuf {
        self.path(cue)
            .as_deref()
            .and_then(Path::parent)
            .unwrap_or(&self.directory)
            .to_owned()
    }

    /// Returns the name by which a cue plays the file at `path`: its path
    /// from the directory that relative paths start from, when the file is
    /// in that directory, so that the show and its files can move together,
    /// or else the path as it is
    pub fn file_name_of(&self, path: &Path) -> String {
        path.strip_prefix(&self.directory)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned()
    }
}
