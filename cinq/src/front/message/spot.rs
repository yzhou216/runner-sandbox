// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Place in the show, which the history of places holds and the messages
//! and the status bar name

use crate::back::{CueId, OutputId};

/// A place in the show.  A cue names its place by its identity, so that the
/// place follows the cue as it moves between cue lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spot {
    Cue(CueId),
    /// Cue list with no selected cue, which only its index names
    List(usize),
    /// Output that the outputs view shows selected, which follows it as the
    /// outputs move
    Output(OutputId),
    /// Outputs view with no selected output
    Outputs,
}
