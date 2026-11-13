// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cue lists and light patch put in place of those of the show as a
//! whole, as bringing back unsaved changes and its undo and redo do

use crate::back::{
    Change, Contents, Workspace,
    workspace::{listed, next_id},
};

impl Workspace {
    pub fn contents(&self) -> Contents {
        Contents {
            lists: self.lists.clone(),
            patch: self.patch.clone(),
        }
    }

    pub(in crate::back::workspace) fn restored(self, contents: Contents) -> Self {
        let before = self.contents();
        self.with_contents(contents.clone())
            .recorded(Change::Restored {
                before,
                after: contents,
            })
    }

    /// Puts cue lists and a patch in place of those of the show without
    /// recording a change, as undo and redo do.  The cues that have gone
    /// stop, and new cues get identities that none of the lists holds.
    pub(in crate::back::workspace) fn with_contents(self, contents: Contents) -> Self {
        let lists = listed(contents.lists);
        let workspace = Self {
            next_id: next_id(&lists).max(self.next_id),
            lists,
            ..self.with_patch(contents.patch)
        };
        let gone = workspace.active_where(|active| workspace.cue(active.id).is_none());
        gone.into_iter().fold(workspace, Self::stopped)
    }
}
