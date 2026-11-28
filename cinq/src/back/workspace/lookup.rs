// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Finding the cue lists, the cues, and the running cues of a workspace

use super::Workspace;
use crate::back::{Active, Cue, CueId, CueList};

impl Workspace {
    pub fn lists(&self) -> &[CueList] {
        &self.lists
    }

    /// Returns the cues that GO has fired and that have not finished yet,
    /// in the order that GO fired them, the latest last
    pub fn active(&self) -> &[Active] {
        &self.active
    }

    pub fn show_mode(&self) -> bool {
        self.show_mode
    }

    /// Returns the identity the next added cue will receive, or none once
    /// the identities have run out
    pub fn next_cue_id(&self) -> Option<CueId> {
        u32::try_from(self.next_id).ok().map(CueId)
    }

    pub fn cues(&self) -> impl Iterator<Item = &Cue> {
        self.lists.iter().flat_map(|list| &list.cues)
    }

    pub fn cue(&self, id: CueId) -> Option<&Cue> {
        self.cues().find(|cue| cue.id == id)
    }

    /// Returns the first cue whose number is `number`, in the order of the
    /// lists and then of the rows, since cue numbers need not be unique.
    /// Surrounding spaces count for nothing, and no cue is the one with no
    /// number.
    pub fn numbered(&self, number: &str) -> Option<&Cue> {
        let number = number.trim();
        self.cues()
            .find(|cue| !number.is_empty() && cue.number.trim() == number)
    }

    pub fn target(&self, cue: &Cue) -> Option<&Cue> {
        cue.target.and_then(|id| self.cue(id))
    }

    pub fn list_of(&self, id: CueId) -> Option<usize> {
        self.lists
            .iter()
            .position(|list| list.position(id).is_some())
    }

    /// Returns the cue list that holds the cue `id`
    pub fn list_holding(&self, id: CueId) -> Option<&CueList> {
        self.list_of(id).and_then(|list| self.lists.get(list))
    }

    pub fn active_of(&self, id: CueId) -> Option<&Active> {
        self.active.iter().find(|active| active.id == id)
    }
}
