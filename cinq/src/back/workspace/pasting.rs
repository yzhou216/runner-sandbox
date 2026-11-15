// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Copying cues out of the show and pasting them into it, which keeps their
//! files and the cues they target right wherever they land

use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use super::{
    Workspace,
    history::{Change, Placed, Standby},
    numbering::Numbers,
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
    /// directory of the show goes by its path from there.  A cue whose
    /// target is among the cues pasted together, in the same round of a
    /// clipboard pasted several times, targets the copy of that cue, as in
    /// QLab.  Otherwise a target is kept when it is a cue of the show and
    /// dropped when it is not.  Once the identities have run out, no cue is
    /// pasted.
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
        let known: HashSet<CueId> = self.cues().map(|cue| cue.id).collect();
        let numbers = Numbers::of(self.cues().map(|cue| cue.number.as_str()));
        let rounds = rounds(&cues);
        let copies: HashMap<(usize, CueId), CueId> = rounds
            .iter()
            .zip(&cues)
            .zip(&ids)
            .map(|((&round, cue), &id)| ((round, cue.id), id))
            .collect();
        let pasted: Vec<Cue> = cues
            .into_iter()
            .zip(&ids)
            .zip(rounds)
            .scan(numbers, |numbers, ((cue, &id), round)| {
                let number = if cue.number.trim().is_empty() || !numbers.holds(&cue.number) {
                    cue.number.clone()
                } else {
                    numbers.next()
                };
                numbers.take(&number);
                let file = cue.file_name().map_or_else(
                    || cue.file.clone(),
                    |name| self.file_name_of(Path::new(name)),
                );
                let target = cue.target.and_then(|target| {
                    copies
                        .get(&(round, target))
                        .copied()
                        .or_else(|| known.contains(&target).then_some(target))
                });
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
        let cues = workspace.lists[list].cues[index..index + ids.len()]
            .iter()
            .enumerate()
            .map(|(offset, cue)| Placed {
                list,
                index: index + offset,
                cue: cue.clone(),
            })
            .collect();
        let standby = Standby {
            list,
            before,
            after: workspace.lists[list].standby,
        };
        workspace.recorded(Change::Pasted { cues, at, standby })
    }
}

/// Returns the round of each cue.  A clipboard pasted several times in a
/// row puts its cues again, so that a cue whose identity came already in
/// the round begins the next.
fn rounds(cues: &[Cue]) -> Vec<usize> {
    cues.iter()
        .scan((0, HashSet::new()), |(round, seen), cue| {
            if !seen.insert(cue.id) {
                *round += 1;
                seen.clear();
                seen.insert(cue.id);
            }
            Some(*round)
        })
        .collect()
}
