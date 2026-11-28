// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Copying cues out of the show and pasting them into it, which keeps their
//! files and the cues they target right wherever they land

mod rounds;

use std::{
    collections::{HashMap, HashSet},
    ops::Range,
    path::Path,
};

use self::rounds::{rounds, shapes};
use super::{
    super::{
        Workspace, files,
        history::{Change, Placed},
    },
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
            .map_or_else(|| cue.file.clone(), |path| files::written(&path));
        Cue {
            file,
            media: Media::Unloaded,
            // Stamp that matched would tell the recheck that nothing needs reading
            seen: None,
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
    /// dropped when it is not.  The cues keep their shape: their depths shift
    /// together, so that the first of them takes the depth that a cue added
    /// there takes, and each round of a clipboard pasted several times
    /// starts at that depth.  Once the identities have run out, no cue is
    /// pasted.
    pub(in crate::back::workspace) fn pasted(self, list: usize, cues: Vec<Cue>, at: Place) -> Self {
        let Some(target) = self.lists.get(list) else {
            return self;
        };
        let ids = self.identities(cues.len());
        if ids.is_empty() || ids.len() < cues.len() {
            return self;
        }
        let (before, index) = (target.standby, target.index_at(at));
        let pasted = self.copies(cues, &ids, target.depth_at(at));
        let workspace = Self {
            next_id: self.next_id + ids.len() as u64,
            ..self.with_list(list, |cues| cues.with_pasted(index, pasted))
        };
        workspace.recorded_paste(list, index..index + ids.len(), before, at)
    }

    /// Returns the copies that `cues` become when they take `ids`, with
    /// numbers, files, targets, and depths as `pasted` describes
    fn copies(&self, cues: Vec<Cue>, ids: &[CueId], depth: u8) -> Vec<Cue> {
        let known: HashSet<CueId> = self.cues().map(|cue| cue.id).collect();
        let numbers = Numbers::of(self.cues().map(|cue| cue.number.as_str()));
        let rounds = rounds(&cues);
        let copies: HashMap<(usize, CueId), CueId> = rounds
            .iter()
            .zip(&cues)
            .zip(ids)
            .map(|((&round, cue), &id)| ((round, cue.id), id))
            .collect();
        let shapes = shapes(&cues, &rounds);
        let (pasted, _) = cues.into_iter().zip(ids).zip(rounds).zip(shapes).fold(
            (Vec::new(), numbers),
            |(mut pasted, numbers), (((cue, &id), round), shape)| {
                let (number, numbers) = numbers.claim(&cue.number);
                let file = cue.file_name().map_or_else(
                    || cue.file.clone(),
                    |name| self.file_name_of(Path::new(name)),
                );
                let target = cue
                    .target
                    .and_then(|target| retargeted(target, round, &copies, &known));
                // Pushed rather than chained, since a large paste would go quadratic
                pasted.push(Cue {
                    id,
                    number,
                    file,
                    target,
                    depth: depth.saturating_add(shape),
                    ..cue
                });
                (pasted, numbers)
            },
        );
        pasted
    }

    fn recorded_paste(
        self,
        list: usize,
        indices: Range<usize>,
        before: Option<CueId>,
        at: Place,
    ) -> Self {
        // Read from the slice, since a search for each identity would make
        // a large paste quadratic
        let cues = self.lists[list].cues[indices.clone()]
            .iter()
            .zip(indices)
            .map(|(cue, index)| Placed {
                list,
                index,
                cue: cue.clone(),
            })
            .collect();
        let standby = self.standby_since(list, before);
        self.recorded(Change::Pasted { cues, at, standby })
    }
}

/// Returns the target of a pasted cue, given the original that it targeted:
/// the copy of it in the same round, or else the original where the show
/// has it, or none
fn retargeted(
    target: CueId,
    round: usize,
    copies: &HashMap<(usize, CueId), CueId>,
    known: &HashSet<CueId>,
) -> Option<CueId> {
    copies
        .get(&(round, target))
        .copied()
        .or_else(|| known.contains(&target).then_some(target))
}
