// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Edits to one cue, and what loading its file found

use super::super::{Workspace, history::Change};
use crate::back::{CueId, Edit, Kind, Media, Stamp};

impl Workspace {
    /// Edits a cue and records the edit, unless it leaves the setting as it
    /// was, when it changes nothing at all.  Text fields send their text
    /// again on keys that change nothing, such as Backspace at the start of
    /// a field, and such an edit must not stop a running cue.
    pub(in crate::back::workspace) fn edited(self, id: CueId, edit: Edit) -> Self {
        let undo = self
            .cue(id)
            .map(|cue| cue.reverting(&edit))
            .filter(|undo| *undo != edit);
        match undo {
            Some(undo) => self.changed(id, edit.clone()).recorded(Change::Edited {
                cue: id,
                undo,
                redo: edit,
            }),
            None => self,
        }
    }

    /// Edits a cue without recording the edit, as undo and redo edit it.
    /// A running cue stops when it gets another file, which has yet to
    /// load, while a file name that leads to the same file, such as the
    /// one chosen again, leaves it playing.  A running audio or video cue
    /// moves its level by the change in the level of its cue, so that a cue
    /// that a fade has lowered stays lower, and a fade under way that leaves
    /// it playing goes on from there to its own level.  A cue that a panic
    /// or a fade is stopping keeps fading out, so that no edit brings it
    /// back.  The level of a fade cue is the level of the fades that the cue
    /// is making, which turn from where they are toward it in the time they
    /// have left, and of its runs that have yet to act.
    pub(in crate::back::workspace) fn changed(self, id: CueId, edit: Edit) -> Self {
        let Some(cue) = self.cue(id) else {
            return self;
        };
        let (path, plays) = (self.path(cue), cue.kind.plays_file());
        let workspace = match edit {
            Edit::Level(level) if plays => {
                let by = level - cue.level;
                self.with_active(id, |active| match active.fade {
                    Some(fade) if fade.then_stop => active,
                    _ => active.moved(by),
                })
            }
            Edit::Level(level) if cue.kind == Kind::Fade => {
                self.with_each_active(|active| active.retargeted(id, level))
            }
            _ => self,
        }
        .with_list_of(id, |cues| cues.edited(id, edit));
        let refiled = workspace
            .cue(id)
            .is_some_and(|cue| workspace.path(cue) != path);
        if refiled {
            workspace
                .stopped(id)
                .with_list_of(id, |cues| cues.loaded(id, Media::Unloaded, None))
        } else {
            workspace
        }
    }

    /// Records what loading the file of a cue found, unless the cue has
    /// been given another file since loading began.  A run whose sound
    /// could not start then ends, and the rest of its chain plays on, as GO
    /// skips a cue that cannot play.
    pub(in crate::back::workspace) fn loaded(
        self,
        id: CueId,
        file: &str,
        media: Media,
        seen: Option<Stamp>,
    ) -> Self {
        if self.cue(id).is_none_or(|cue| cue.file != file) {
            return self;
        }
        let workspace = match media {
            Media::Failed(_) => self.skipped(id),
            _ => self,
        };
        workspace.with_list_of(id, |cues| cues.loaded(id, media, seen))
    }
}
