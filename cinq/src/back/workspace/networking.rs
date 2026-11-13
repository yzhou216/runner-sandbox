// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rules of network cues, which send their message as they start
//!
//! A network cue that starts puts its message in the outbox of the
//! workspace, which the network output empties after each change, so that
//! the message goes out once for each time the cue runs.

use std::mem;

use super::Workspace;
use crate::back::{CueId, Destination, Outgoing, osc};

impl Workspace {
    /// Returns the workspace with the message of network cue `id` in its
    /// outbox, as the cue reads when it starts
    pub(super) fn sent_by(self, id: CueId) -> Self {
        let outgoing = self.cue(id).and_then(|cue| {
            Some(Outgoing {
                destination: Destination::parse(&cue.destination)?,
                packet: osc::packet(&cue.message)?,
            })
        });
        Self {
            outbox: self.outbox.into_iter().chain(outgoing).collect(),
            ..self
        }
    }

    /// Returns the workspace with its outbox empty, together with the
    /// messages that were in it, for the network output to send
    pub fn drained(mut self) -> (Self, Vec<Outgoing>) {
        let outbox = mem::take(&mut self.outbox);
        (self, outbox)
    }
}
