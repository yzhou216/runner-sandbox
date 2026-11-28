// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rules of network cues, which send their message as they start
//!
//! A network cue that starts puts its message in the outbox of the
//! workspace, which the network output empties after each change, so that
//! the message goes out once for each run of the cue.

use std::collections::{BTreeMap, BTreeSet};

use itertools::Itertools;

use crate::back::{Cue, CueId, Destination, Kind, Outgoing, Workspace, osc};

impl Workspace {
    /// Puts the message of network cue `id` in the outbox, unless the cue
    /// has no destination or message that could be read
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

    /// Returns the destinations that the network cues name, as the network
    /// output keeps its problems by them, each with the first cue that names
    /// it, which a problem of the destination is told for
    pub fn destinations(&self) -> BTreeMap<String, CueId> {
        self.cues()
            .filter_map(|cue| Some((destination_of(cue)?, cue.id)))
            .unique_by(|(destination, _)| destination.clone())
            .collect()
    }

    /// Returns whether `cue` is a network cue whose destination is among
    /// `failed`, the destinations that messages last failed to reach
    pub fn sends_to_failed(&self, cue: &Cue, failed: &BTreeSet<String>) -> bool {
        destination_of(cue).is_some_and(|destination| failed.contains(&destination))
    }

    /// Returns the workspace with an empty outbox, and the messages that the
    /// outbox held
    pub fn drained(self) -> (Self, Vec<Outgoing>) {
        let outbox = self.outbox;
        (
            Self {
                outbox: Vec::new(),
                ..self
            },
            outbox,
        )
    }
}

/// Returns the destination of a network cue, as the network output names it
fn destination_of(cue: &Cue) -> Option<String> {
    (cue.kind == Kind::Network)
        .then(|| Destination::parse(&cue.destination))
        .flatten()
        .map(|destination| destination.to_string())
}
