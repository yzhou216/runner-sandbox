// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Requests that change a workspace

use crate::back::{CueId, CueList, Edit, Instrument, Kind, Media, PatchEdit};

/// Request to change the show
///
/// The interface sends these through [`super::Workspace::apply`].  Other
/// front ends, such as an OSC server, could send the same commands.
#[derive(Debug, Clone)]
pub enum Command {
    /// Fires the standby cue of a list along with its chain
    Go { list: usize },
    /// Fades out every playing cue and stops the rest, or stops everything
    /// at once while an earlier panic is still fading
    Panic,
    /// Pauses or resumes every running cue
    Pause(bool),
    /// Stops one running cue
    Stop(CueId),
    /// Advances the playback clock by a number of seconds
    Tick(f64),
    /// Moves the standby position of a list
    Standby { list: usize, cue: Option<CueId> },
    /// Locks editing for a performance, or unlocks it
    ShowMode(bool),
    /// Adds a cue of a kind at a place in a list
    Add { list: usize, kind: Kind, at: Place },
    /// Removes cues, stopping those that are running, as one change
    Delete(Vec<CueId>),
    /// Moves a cue within its list by a number of places, downwards when
    /// positive, as far as the ends of the list allow
    Move { cue: CueId, by: isize },
    /// Changes one setting of a cue
    Edit(CueId, Edit),
    /// Changes the light patch
    Patch(PatchEdit),
    /// Puts cue lists and a patch kept from before in place of those of
    /// the show, as one change, as bringing back unsaved changes does
    Restore(Contents),
    /// Takes back the last change to the cues
    Undo,
    /// Makes again the change that undo took back last
    Redo,
    /// Moves the clock of a run toward the seconds of its clip that its
    /// sound has played, as the audio device keeps time
    Heard { cue: CueId, run: u32, played: f64 },
    /// Records what loading the file of a cue found
    Loaded {
        cue: CueId,
        file: String,
        media: Media,
    },
}

impl Command {
    /// Returns whether the command edits the show, which show mode forbids
    pub fn edits(&self) -> bool {
        matches!(
            self,
            Command::Add { .. }
                | Command::Delete(_)
                | Command::Move { .. }
                | Command::Edit(..)
                | Command::Patch(_)
                | Command::Restore(_)
        )
    }
}

/// Cue lists and light patch of a show, which bringing back its unsaved
/// changes puts in place together
#[derive(Debug, Clone, Default)]
pub struct Contents {
    pub lists: Vec<CueList>,
    pub patch: Vec<Instrument>,
}

/// Where in its list a new cue goes
#[derive(Debug, Clone, Copy)]
pub enum Place {
    After(CueId),
    Before(CueId),
    /// End of the list, which is also where a cue goes whose neighbour is
    /// not in the list
    End,
}
