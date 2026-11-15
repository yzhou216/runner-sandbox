// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Requests that change a workspace

use crate::back::{Cue, CueId, CueList, Edit, Kind, Media, Output, OutputEdit, OutputId};

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
    /// Adds a number of cues of a kind in a row at a place in a list, as one
    /// change
    Add {
        list: usize,
        kind: Kind,
        at: Place,
        count: usize,
    },
    /// Pastes copies of cues, in order, at a place in a list as one change,
    /// each with an identity of its own
    Paste {
        list: usize,
        cues: Vec<Cue>,
        at: Place,
    },
    /// Removes cues, stopping those that are running, as one change
    Delete(Vec<CueId>),
    /// Removes cues as [`Command::Delete`] does, and records that they went
    /// to the clipboard, so that the change is told as a cut
    Cut(Vec<CueId>),
    /// Moves a cue within its list by a number of places, downwards when
    /// positive, as far as the ends of the list allow
    Move { cue: CueId, by: isize },
    /// Changes one setting of a cue
    Edit(CueId, Edit),
    /// Puts cue lists and outputs kept from before in place of those of
    /// the show, as one change, as bringing back unsaved changes does
    Restore(Vec<CueList>, Vec<Output>),
    /// Adds a number of outputs in a row at a place among the outputs, counted
    /// from zero, or at the end past them, as one change
    AddOutput { at: usize, count: usize },
    /// Removes outputs as one change, unless that leaves none
    DeleteOutputs(Vec<OutputId>),
    /// Removes outputs as [`Command::DeleteOutputs`] does, and records that
    /// they went to the register of outputs, so that the change is told as a
    /// cut
    CutOutputs(Vec<OutputId>),
    /// Pastes outputs in a row at a place among the outputs as one change.
    /// With `restore`, an output whose identity is free takes it back, so that
    /// the cues that sent to it send to it again.
    PasteOutputs {
        at: usize,
        outputs: Vec<Output>,
        restore: bool,
    },
    /// Moves an output by a number of places, downwards when positive
    MoveOutput { output: OutputId, by: isize },
    /// Changes one setting of an output
    EditOutput(OutputId, OutputEdit),
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
                | Command::Paste { .. }
                | Command::Delete(_)
                | Command::Cut(_)
                | Command::Move { .. }
                | Command::Edit(..)
                | Command::Restore(..)
                | Command::AddOutput { .. }
                | Command::DeleteOutputs(_)
                | Command::CutOutputs(_)
                | Command::PasteOutputs { .. }
                | Command::MoveOutput { .. }
                | Command::EditOutput(..)
        )
    }
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
