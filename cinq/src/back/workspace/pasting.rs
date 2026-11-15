// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Copying cues out of the show and pasting them into it, which keeps their
//! files and the cues they target right wherever they land

use std::path::Path;

use super::{
    Workspace,
    editing::next_number,
    history::{Change, Standby},
};
use crate::back::{Cue, CueId, Media, Place};

impl Workspace {
    /// Returns a copy of `cue` that another show can take in: its file by
    /// the path it has from the filesystem root, since a relative path goes
    /// from the directory of its own show, and without what loading found
    pub fn copied(&self, cue: &Cue) -> Cue {
        let file = self
            .path(cue)
            .map_or_else(|| cue.file.clone(), |path| path.to_string_lossy().into());
        Cue {
            file,
            media: Media::Unloaded,
            ..cue.clone()
        }
    }

    /// Pastes copies of cues into a list at a place as one change, each with
    /// an identity of its own.  A cue keeps its number unless another cue
    /// holds it, when it takes the next free number, so that a cue that was
    /// cut keeps its number where it is pasted.  A file inside the
    /// directory of the show goes by its path from there, and a target that
    /// is no cue of the show is dropped.  Once the identities have run out,
    /// no cue is pasted.
    pub(super) fn pasted(self, list: usize, cues: Vec<Cue>, at: Place) -> Self {
        let Some(target) = self.lists.get(list) else {
            return self;
        };
        let ids: Option<Vec<CueId>> = (self.next_id..)
            .take(cues.len())
            .map(|id| u32::try_from(id).ok().map(CueId))
            .collect();
        let Some(ids) = ids.filter(|ids| !ids.is_empty()) else {
            return self;
        };
        let (before, index) = (target.standby, target.index_at(at));
        let taken = self.cues().map(|cue| cue.number.trim().to_owned());
        let pasted: Vec<Cue> = cues
            .into_iter()
            .zip(&ids)
            .scan(taken.collect::<Vec<_>>(), |taken, (cue, &id)| {
                let held = taken.iter().any(|taken| taken == cue.number.trim());
                let number = if cue.number.trim().is_empty() || !held {
                    cue.number.clone()
                } else {
                    next_number(taken.iter().map(String::as_str))
                };
                taken.push(number.trim().to_owned());
                let file = cue.file_name().map_or_else(
                    || cue.file.clone(),
                    |name| self.file_name_of(Path::new(name)),
                );
                let target = cue.target.filter(|&target| self.cue(target).is_some());
                Some(Cue {
                    id,
                    number,
                    file,
                    target,
                    ..cue
                })
            })
            .collect();
        let workspace = Self {
            next_id: self.next_id + ids.len() as u64,
            ..self.with_list(list, |cues| cues.with_pasted(index, pasted))
        };
        let cues = ids
            .iter()
            .filter_map(|&id| workspace.place_of(id))
            .collect();
        let standby = Standby {
            list,
            before,
            after: workspace.lists[list].standby,
        };
        workspace.recorded(Change::Pasted { cues, standby })
    }
}
