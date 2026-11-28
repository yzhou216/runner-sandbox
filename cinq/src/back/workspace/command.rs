// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Requests that change a workspace

use crate::back::{
    Cue, CueId, CueList, Edit, Instrument, Kind, Media, Output, OutputEdit, OutputId, PatchEdit,
    Stamp,
};

/// Request to change the show
///
/// The interface sends these through [`super::Workspace::apply`], for the
/// keys and buttons as well as for the commands that arrive over OSC.
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
    /// Fires a cue with its chain, or resumes it where it is paused, as a
    /// start cue does
    Start(CueId),
    /// Holds the run of a cue, as a pause cue does
    Hold(CueId),
    /// Resumes the run of a cue that is held, as a start cue does
    Resume(CueId),
    /// Moves standby to a cue in the list that holds it, as a GoTo cue does
    GoTo(CueId),
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
    /// Removes cues, stopping those that are running, as one change, each
    /// group together with the cues that it holds
    Delete(Vec<CueId>),
    /// Removes cues as [`Command::Delete`] does, and records that they went
    /// to the clipboard, so that the change is told as a cut
    Cut(Vec<CueId>),
    /// Moves a cue within its list by a number of places, downwards when
    /// positive, as far as the ends of the list allow, a group together with
    /// the cues that it holds
    Move { cue: CueId, by: isize },
    /// Moves cues into the group above each of them, with the cues that
    /// each holds, as one change
    Indent(Vec<CueId>),
    /// Moves cues out of the group that holds each of them, with the cues
    /// that each holds, as one change
    Outdent(Vec<CueId>),
    /// Collapses the groups among the cues, or expands them, which is no
    /// edit and no change to undo, so that show mode allows it
    Collapse { cues: Vec<CueId>, collapsed: bool },
    /// Changes one setting of a cue
    Edit(CueId, Edit),
    /// Changes the light patch
    Patch(PatchEdit),
    /// Puts cue lists, outputs, and a light patch kept from before in
    /// place of those of the show, as one change, as bringing back unsaved
    /// changes does
    Restore(Vec<CueList>, Vec<Output>, Vec<Instrument>),
    /// Adds a number of outputs in a row at a place among the outputs, counted
    /// from zero, or at the end past them, as one change, next to the output
    /// `anchor` that was selected, which undo selects again
    AddOutput {
        at: usize,
        count: usize,
        anchor: Option<OutputId>,
    },
    /// Removes outputs as one change, unless that leaves none
    DeleteOutputs(Vec<OutputId>),
    /// Removes outputs as [`Command::DeleteOutputs`] does, and records that
    /// they went to the register of outputs, so that the change is told as a
    /// cut
    CutOutputs(Vec<OutputId>),
    /// Pastes outputs in a row at a place among the outputs as one change.
    /// With `restore`, an output whose identity is free takes it back, so that
    /// the cues that sent to it send to it again.  The `anchor` is the output
    /// that was selected, as for [`Command::AddOutput`].
    PasteOutputs {
        at: usize,
        outputs: Vec<Output>,
        restore: bool,
        anchor: Option<OutputId>,
    },
    /// Moves an output by a number of places, downwards when positive
    MoveOutput { output: OutputId, by: isize },
    /// Changes one setting of an output
    EditOutput(OutputId, OutputEdit),
    /// Ends the run of edits to one setting in a row, so that the next edit
    /// of it makes a change of its own, as leaving a field does
    EndRun,
    /// Takes back the last change to the cues
    Undo,
    /// Makes again the change that undo took back last
    Redo,
    /// Moves the clock of a run toward the seconds of its clip that its
    /// sound has played, as the audio device keeps time
    Heard { cue: CueId, run: u32, played: f64 },
    /// Records what loading the file of a cue found, along with the stamp
    /// that the file had then, from which the next look at it tells whether
    /// it has changed
    Loaded {
        cue: CueId,
        file: String,
        media: Media,
        seen: Option<Stamp>,
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
                | Command::Indent(_)
                | Command::Outdent(_)
                | Command::Edit(..)
                | Command::Patch(_)
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

#[cfg(test)]
impl Command {
    pub fn add_output(at: usize, count: usize) -> Self {
        Self::AddOutput {
            at,
            count,
            anchor: None,
        }
    }
}
