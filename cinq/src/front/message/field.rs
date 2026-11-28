// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Fields that take typed text or the keyboard focus: the settings whose
//! text waits for the operator, where the caret lands, and where a key
//! moves the focus

use crate::back::{CueId, InstrumentId, OutputId};

/// Setting whose field shows the text typed into it, which applies once it
/// reads as a value, or for a level, the file of a cue, or the channels of an
/// output, once the operator has left the field
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setting {
    /// File that an audio, video, or picture cue plays or shows
    File(CueId),
    /// Level of an audio or video cue, or the level that a fade cue fades
    /// its target to
    Level(CueId),
    /// Level at which a cue sends to an output, which an empty field takes
    /// away
    Send(CueId, OutputId),
    OutputName(OutputId),
    Channels(OutputId),
    /// Level at which new cues send to an output, which an empty field
    /// takes away
    NewCues(OutputId),
    /// Level in percent of an instrument in a light cue, which an empty
    /// field leaves unchanged
    Light(CueId, InstrumentId),
    Address(InstrumentId),
}

impl Setting {
    /// Returns whether the text typed into the field waits until the
    /// operator has left it, since a level typed so far, such as the `2` of
    /// `-25`, reads as another level that a playing cue would follow, a path
    /// typed so far reads as a file that does not exist and would stop the
    /// cue, the channels of an output would open a device, and an address
    /// typed so far, such as the `2/5` of `2/500`, would move the instrument
    /// that sends to a rig
    pub(in crate::front) fn waits(self) -> bool {
        !matches!(self, Self::OutputName(_))
    }
}

/// Where the caret lands in a field that a key focuses, and what it selects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Caret {
    End,
    Start,
    /// Every character selected, so that typing replaces them
    All,
}

/// Text field that keys move the keyboard focus to, since iced itself
/// moves it only on clicks
#[derive(Debug, Clone)]
pub enum Focus {
    /// Name field of the inspector, for editing the selected cue, with the
    /// caret where the operator asked for it
    Name(Caret),
    /// Next field, or the first one when no field has focus
    Next,
    Previous,
    /// No field, which hands the keys back to the cue list
    Cues,
}
