// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Loop of the light output thread, one round for each universe change, each
//! frame of a leaving universe, and each quiet second

use std::{
    io,
    net::UdpSocket,
    sync::mpsc::{self, RecvTimeoutError},
    time::{Duration, Instant},
};

use uuid::Uuid;

use super::{
    Interface, Universe, Universes, artnet,
    binding::{self, bound},
    leaving::Leaving,
    listing::Listing,
    reports::Reporter,
    sacn,
    status::Round,
};
use crate::{
    back::{TICK, patch::DARK},
    warn,
};

/// Time after which unchanged levels go out again
const REFRESH: Duration = Duration::from_secs(1);

/// How each send of a round went
type Sends = Vec<io::Result<usize>>;

struct Frame<'a> {
    socket: &'a UdpSocket,
    interfaces: &'a [Interface],
    cid: Uuid,
    sequence: u8,
}

impl Frame<'_> {
    /// Sends the packets of a universe on every interface and returns how
    /// each send went
    fn everywhere(&self, universe: u16, artnet: Option<&[u8]>, sacn: &[u8]) -> Sends {
        self.interfaces
            .iter()
            .flat_map(|interface| interface.send(self.socket, universe, artnet, sacn))
            .collect()
    }

    /// Sends the levels of `channels` as Art-Net and as sACN, which ends
    /// the stream of the universe when `terminated`
    fn levels(&self, universe: u16, channels: &Universe, terminated: bool) -> Sends {
        self.everywhere(
            universe,
            artnet::packet(universe, channels).as_deref(),
            &sacn::packet(self.cid, universe, self.sequence, channels, terminated),
        )
    }

    fn steady(&self, latest: &Universes) -> Sends {
        latest
            .iter()
            .flat_map(|(&universe, channels)| self.levels(universe, channels, false))
            .collect()
    }

    fn ending(&self, leaving: &Leaving) -> Sends {
        leaving
            .frame()
            .flat_map(|(universe, terminated)| self.levels(universe, &DARK, terminated))
            .collect()
    }

    /// Sends the list of universes when the patch has some and the last
    /// list is old, and returns how each send went with the time of the
    /// last list
    fn listed(&self, latest: &Universes, announced: Option<Instant>) -> (Sends, Option<Instant>) {
        if !latest.is_empty() && announced.is_none_or(|at| at.elapsed() >= sacn::DISCOVERY_INTERVAL)
        {
            let universes: Vec<u16> = latest.keys().copied().collect();
            let announced = Some(Instant::now());
            let sends = sacn::discovery(self.cid, &universes)
                .iter()
                .flat_map(|page| self.everywhere(sacn::DISCOVERY, None, page))
                .collect();
            (sends, announced)
        } else {
            (Vec::new(), announced)
        }
    }
}

/// Sends universes as they arrive and the latest again after each quiet
/// second, until the sender is dropped.  A universe that has left the
/// patch goes out as zeros, then as a terminated stream, at the pace of the
/// workspace.  Each change in whether any light gets out goes to `reports`,
/// and once to standard error.
pub(super) fn send(levels: &mpsc::Receiver<Universes>, reports: &Reporter) {
    let Some((socket, first)) = bound(levels, reports, binding::open, binding::RETRY) else {
        return;
    };
    // Random for each run, so that receivers tell sources apart
    let cid = Uuid::new_v4();
    let mut latest = first;
    let mut leaving = Leaving::default();
    let mut listing = Listing::new();
    let mut announced = None::<Instant>;
    let mut problem = None::<String>;
    // One count for each round, since sACN numbers the packets of each
    // universe in order
    for sequence in (0..=u8::MAX).cycle() {
        let Some(received) = receive(levels, latest, leaving) else {
            return;
        };
        (latest, leaving) = received;
        listing = listing.at(Instant::now(), super::interfaces);
        let interfaces = listing.items();
        let due = !latest.is_empty() || !leaving.is_empty();
        let frame = Frame {
            socket: &socket,
            interfaces,
            cid,
            sequence,
        };
        let steady = frame.steady(&latest);
        let ending = frame.ending(&leaving);
        leaving = leaving.next();
        let (listed, last) = frame.listed(&latest, announced);
        announced = last;
        let sends: Sends = steady.into_iter().chain(ending).chain(listed).collect();
        problem = report(reports, &Round::of(interfaces.len(), due, &sends), problem);
    }
}

/// Waits for the next change of the patch, or for the pace of the round to
/// pass, and returns the universes and the leaving ones that the round
/// starts with, or none once the sender is dropped
fn receive(
    levels: &mpsc::Receiver<Universes>,
    latest: Universes,
    leaving: Leaving,
) -> Option<(Universes, Leaving)> {
    let wait = if leaving.is_empty() { REFRESH } else { TICK };
    match levels.recv_timeout(wait) {
        Ok(universes) => {
            let leaving = leaving.changed(&latest, &universes);
            Some((universes, leaving))
        }
        Err(RecvTimeoutError::Timeout) => Some((latest, leaving)),
        Err(RecvTimeoutError::Disconnected) => None,
    }
}

/// Tells what the round changed in the problem, if it did, and returns the
/// problem that now stands
fn report(reports: &Reporter, round: &Round, before: Option<String>) -> Option<String> {
    match round.told(before.as_deref()) {
        Some(now) => {
            tell(reports, now.clone());
            now
        }
        None => before,
    }
}

/// Hands the sidebar what changed in whether light gets out, and standard
/// error the problem, once
pub(super) fn tell(reports: &Reporter, problem: Option<String>) {
    if let Some(words) = &problem {
        warn::once(&format!("light output: {words}"));
    }
    // Receiver has gone only as cinq ends
    let _ = reports.unbounded_send(problem);
}
