// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the files of cues are, which relative file names find from the
//! directory of the show

use std::{
    fs,
    path::{Component, Path, PathBuf},
};

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

    /// Returns the path of the file of `cue`, its parts joined by the
    /// separator of the platform, since a relative name joins them with a
    /// slash, which Windows would leave among its backslashes
    pub(in crate::back) fn path(&self, cue: &Cue) -> Option<PathBuf> {
        cue.file_name().map(|name| {
            self.directory
                .join(Path::new(name).components().collect::<PathBuf>())
        })
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
    /// or else the path as it is.  A directory reached through `..` or a
    /// link is compared as the filesystem resolves it when the plain
    /// comparison fails, since the file chooser may give the path of the
    /// file resolved.  The file itself keeps its name, since a link to a file
    /// elsewhere is still a file of the show.
    pub fn file_name_of(&self, path: &Path) -> String {
        written(
            &path
                .strip_prefix(&self.directory)
                .map(Path::to_owned)
                .ok()
                .or_else(|| self.inside(path))
                .unwrap_or_else(|| path.to_owned()),
        )
    }

    /// Returns the path of `path` from the directory that relative paths
    /// start from, once the directory and the directory of `path` go by the
    /// paths that the filesystem gives them
    fn inside(&self, path: &Path) -> Option<PathBuf> {
        let directory = fs::canonicalize(&self.directory).ok()?;
        let parent = fs::canonicalize(path.parent()?).ok()?;
        let name = path.file_name()?;
        parent
            .strip_prefix(directory)
            .ok()
            .map(|inside| inside.join(name))
    }
}

/// Returns `path` as text for a cue to name its file by.  A relative path
/// joins its components with a slash, which every platform reads, where
/// the separator of Windows would make one file name of it elsewhere.  An
/// absolute path stays as it is.
pub(super) fn written(path: &Path) -> String {
    let absolute = path
        .components()
        .any(|part| matches!(part, Component::Prefix(_) | Component::RootDir));
    if absolute {
        path.to_string_lossy().into_owned()
    } else {
        path.components()
            .map(|part| part.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/")
    }
}
