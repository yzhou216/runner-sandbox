// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cue lists, each with its own standby position

mod groups;
mod placing;

pub use groups::Groups;

use std::collections::HashSet;

use crate::back::{Cue, CueId, Edit, Kind, Media, Place, Stamp, items::replaced_where};

#[derive(Debug, Clone)]
pub struct CueList {
    pub name: String,
    pub cues: Vec<Cue>,
    /// Cue the next GO fires, or `None` past the end of the list
    pub standby: Option<CueId>,
}

impl CueList {
    /// Creates a list with standby on its first cue
    pub fn new(name: impl Into<String>, cues: Vec<Cue>) -> Self {
        Self {
            name: name.into(),
            standby: cues.first().map(|cue| cue.id),
            cues,
        }
    }

    pub fn position(&self, id: CueId) -> Option<usize> {
        self.cues.iter().position(|cue| cue.id == id)
    }

    pub(in crate::back) fn after(&self, id: CueId) -> Option<CueId> {
        self.position(id)
            .and_then(|index| self.cues.get(index + 1))
            .map(|cue| cue.id)
    }

    /// Moves standby to a cue, treating a cue of another list as the end
    pub(in crate::back) fn with_standby(self, standby: Option<CueId>) -> Self {
        let standby = standby.filter(|&id| self.position(id).is_some());
        Self { standby, ..self }
    }

    /// Inserts a new cue of `kind` at `at`, named as new cues of its kind
    /// are.  As in QLab, a new cue that acts on another cue targets the cue
    /// it follows when it can act on that cue, but never a group that holds
    /// the new cue, which a cue added below a group is in.  The cue it
    /// follows is the row above it in the cue list, so that a cue added
    /// below a collapsed group targets the group.
    pub(in crate::back) fn with_new_cue(
        self,
        id: CueId,
        number: String,
        kind: Kind,
        at: Place,
        depth: u8,
    ) -> Self {
        let index = self.index_at(at);
        let cue = Cue::new(id, number, kind, Cue::new_name(kind))
            .depth(depth)
            .build();
        let groups = self.groups();
        let target = index
            .checked_sub(1)
            .and_then(|before| groups.shown_as().get(before).copied())
            .and_then(|row| Some((row, self.cues.get(row)?)))
            .filter(|&(row, before)| {
                cue.can_target(before)
                    && !(before.kind == Kind::Group && depth > groups.depths()[row])
            })
            .map(|(_, before)| before.id);
        self.with_pasted(index, vec![Cue { target, ..cue }])
    }

    /// Inserts cues at an index.  Standby waiting past the end moves to the
    /// first cue inserted at the end, so that GO fires the first cue of a
    /// new list, and a cue added to a list played to its end.
    pub(in crate::back) fn with_pasted(self, index: usize, pasted: Vec<Cue>) -> Self {
        let index = index.min(self.cues.len());
        let standby = self.standby.or(pasted
            .first()
            .filter(|_| index == self.cues.len())
            .map(|cue| cue.id));
        let mut cues = self.cues;
        cues.splice(index..index, pasted);
        Self {
            cues,
            standby,
            ..self
        }
    }

    /// Removes cues in one pass.  Standby on a removed cue moves on to the
    /// next cue that stays, or past the end after the last cue, so that GO
    /// never goes back to a cue that has played.
    pub(in crate::back) fn without(self, gone: &HashSet<CueId>) -> Self {
        let standby = match self.standby {
            Some(id) if gone.contains(&id) => self.position(id).and_then(|row| {
                self.cues[row..]
                    .iter()
                    .find(|cue| !gone.contains(&cue.id))
                    .map(|cue| cue.id)
            }),
            standby => standby,
        };
        Self {
            cues: self
                .cues
                .into_iter()
                .filter(|cue| !gone.contains(&cue.id))
                .collect(),
            standby,
            ..self
        }
    }

    /// Moves a cue to an index, or to the end when the list is shorter,
    /// leaving standby where it is
    pub(in crate::back) fn with_moved(self, id: CueId, to: usize) -> Self {
        let (moving, rest): (Vec<Cue>, Vec<Cue>) =
            self.cues.into_iter().partition(|cue| cue.id == id);
        moving
            .into_iter()
            .fold(Self { cues: rest, ..self }, |list, cue| {
                list.with_cue_at(to, cue)
            })
    }

    pub(in crate::back) fn with_cue_at(self, index: usize, cue: Cue) -> Self {
        let mut cues = self.cues;
        cues.insert(index.min(cues.len()), cue);
        Self { cues, ..self }
    }

    pub(in crate::back) fn edited(self, id: CueId, edit: Edit) -> Self {
        self.with_cue(id, |cue| cue.edited(edit))
    }

    pub(in crate::back) fn loaded(self, id: CueId, media: Media, seen: Option<Stamp>) -> Self {
        self.with_cue(id, |cue| Cue { media, seen, ..cue })
    }

    fn with_cue(self, id: CueId, change: impl FnOnce(Cue) -> Cue) -> Self {
        Self {
            cues: replaced_where(self.cues, |cue| cue.id == id, change),
            ..self
        }
    }
}
