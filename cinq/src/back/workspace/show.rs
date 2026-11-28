// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Show files, which hold the outputs, the light patch, and the cue lists of
//! a workspace as TOML
//!
//! A show file lists the outputs, then the instruments of the light patch,
//! then the cue lists in order, each with its cues.  A cue holds the
//! settings that its kind uses, leaving out those that keep their defaults,
//! apart from the level, the duration, and whether a fade stops its target,
//! which it holds for every cue whose kind uses them.
//! Cues and instruments keep their identities, by which fade and stop cues
//! name their targets and light cues their instruments.  What playback and
//! loading find, such as the cue on standby, the cues that run, the light
//! levels, and how long each sound or video file plays, belongs to the
//! session rather than to the show, and the file leaves it out.

mod check;
mod cue;
mod outputs;
mod syntax;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{Workspace, files};
use crate::back::{Cue, CueList, Instrument, Output};
use cue::CueFile;
use outputs::OutputFile;
use syntax::read_problem;

#[derive(Serialize, Deserialize)]
struct ShowFile {
    #[serde(default, rename = "output", skip_serializing_if = "Vec::is_empty")]
    outputs: Vec<OutputFile>,
    #[serde(default, rename = "instrument", skip_serializing_if = "Vec::is_empty")]
    instruments: Vec<Instrument>,
    #[serde(default, rename = "list")]
    lists: Vec<ListFile>,
}

#[derive(Serialize, Deserialize)]
struct ListFile {
    name: String,
    #[serde(default, rename = "cue", skip_serializing_if = "Vec::is_empty")]
    cues: Vec<CueFile>,
}

impl Workspace {
    /// Reads a show from the text of its file, finding the files of its
    /// cues from `directory`.  Each list starts with standby on its
    /// first cue, and a file without lists holds one empty list, as every
    /// show does.  A file in which two cues or two instruments share an
    /// identity cannot be read, nor can one with an address that no
    /// instrument could be given, or a level that no cue could, since a
    /// level typed wrong by hand, such as 60 for -60, would play at a
    /// thousand times the amplitude of its file.
    pub fn from_show(text: &str, directory: impl Into<PathBuf>) -> Result<Self, String> {
        let show: ShowFile = toml::from_str(text).map_err(|error| read_problem(text, &error))?;
        let outputs = outputs::read(show.outputs)?;
        let lists = show
            .lists
            .into_iter()
            .map(|list| list.list(&outputs))
            .collect::<Result<_, _>>()?;
        let workspace = Self::new(lists)
            .directory(directory)
            .with_outputs(outputs)
            .with_patch(show.instruments);
        check::problem(&workspace).map_or(Ok(workspace), Err)
    }

    /// Returns the text of the show as a file in `directory` holds it, with
    /// the relative paths of the files of cues going from that directory,
    /// or as absolute paths without one
    pub fn show(&self, directory: Option<&Path>) -> String {
        let show = ShowFile {
            outputs: outputs::written(&self.outputs),
            instruments: self.patch.clone(),
            lists: self
                .lists
                .iter()
                .map(|list| ListFile {
                    name: list.name.clone(),
                    cues: list
                        .cues
                        .iter()
                        .map(|cue| {
                            CueFile::of(cue, self.written_file(cue, directory), &self.outputs)
                        })
                        .collect(),
                })
                .collect(),
        };
        toml::to_string(&show).expect("TOML holds every setting of a show")
    }

    /// Returns the file that `cue` plays as a show file in `directory`
    /// names it: an absolute path as it is, and a relative one from
    /// `directory` rather than from the directory of the workspace, or as
    /// an absolute path without one
    fn written_file(&self, cue: &Cue, directory: Option<&Path>) -> String {
        let Some(name) = cue.file_name().filter(|_| cue.kind.has_file()) else {
            return String::new();
        };
        if Path::new(name).is_absolute() || directory == Some(self.directory.as_path()) {
            return name.to_owned();
        }
        let absolute = self.directory.join(name);
        files::written(
            &directory
                .and_then(|directory| pathdiff::diff_paths(&absolute, directory))
                .unwrap_or(absolute),
        )
    }
}

impl ListFile {
    fn list(self, outputs: &[Output]) -> Result<CueList, String> {
        let cues = self
            .cues
            .into_iter()
            .map(|cue| cue.cue(outputs))
            .collect::<Result<_, _>>()?;
        Ok(CueList::new(self.name, cues))
    }
}
