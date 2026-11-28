// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cues that send only to outputs that the show no longer has
//!
//! Deleting or cutting an output leaves the sends to it in the cues, so that
//! undo can bring them back, and a cue whose sends all name such outputs plays
//! on none.  A cue that names no sends plays on the first output, and a cue
//! whose sends were all cleared plays nowhere by choice, as a video cue does
//! that shows a picture and no sound, so that neither counts.

use crate::back::{Cue, Media, Output, OutputId, Workspace};

impl Workspace {
    /// Returns whether `cue` plays a file and sends to outputs, none of which
    /// the show has
    pub fn sends_nowhere(&self, cue: &Cue) -> bool {
        stranded(cue, &self.outputs)
    }

    /// Returns whether `cue` plays a file on outputs of the show that all lie
    /// among `failing`, which makes no sound.  A cue that names no sends plays
    /// on the first output, and a video cue whose file holds no sound has
    /// none to lose, since its picture still shows.
    pub fn sends_to_failing(&self, cue: &Cue, failing: &[OutputId]) -> bool {
        let sent = self.sends(cue);
        let silent = matches!(cue.media, Media::Loaded { sound: None, .. });
        cue.kind.plays_file()
            && !silent
            && !sent.is_empty()
            && sent.keys().all(|id| failing.contains(id))
    }

    /// Returns the cues that send to no output that the show has
    pub fn stranded(&self) -> impl Iterator<Item = &Cue> {
        self.cues().filter(|cue| self.sends_nowhere(cue))
    }

    /// Counts the cues that would send to no output among `outputs`, which
    /// tells what a change to the outputs did to them
    pub fn stranded_among(&self, outputs: &[Output]) -> usize {
        self.cues().filter(|cue| stranded(cue, outputs)).count()
    }
}

fn stranded(cue: &Cue, outputs: &[Output]) -> bool {
    cue.kind.plays_file()
        && cue.sends.as_ref().is_some_and(|sends| {
            !sends.is_empty()
                && sends
                    .keys()
                    .all(|id| outputs.iter().all(|output| output.id != *id))
        })
}
