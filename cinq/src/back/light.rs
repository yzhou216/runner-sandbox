// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Light output, where the back end has side effects
//!
//! [`Lights::sync`] runs after every change to the workspace, as
//! [`super::Audio::sync`] does, and hands the DMX levels of the light patch
//! to a thread of its own whenever they change.  The thread sends each
//! universe as Art-Net to the broadcast address of every network that the
//! computer is on, and as sACN to the multicast group of the universe on
//! each of them, when it changes and again each second, so that a node that
//! starts late or loses a packet catches up.  Every ten seconds it also
//! lists its universes, as sACN asks of a source.  A show without
//! instruments sends nothing.
//!
//! A universe that leaves the patch, by an edit or by the opening of another
//! show, goes out as zeros for a few frames and then as a stream that says
//! that it has ended, so that no receiver keeps its last levels.
//!
//! When cinq ends, its sACN stream does not say that it has ended, since
//! receivers would drop the lights at once.  They hold the levels instead,
//! as they do for any source that has gone quiet.

mod artnet;
mod leaving;
mod listing;
mod reports;
mod sacn;
mod sending;
mod status;
#[cfg(test)]
mod tests;

use std::{
    collections::BTreeMap,
    io,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    sync::mpsc,
    thread,
};

use if_addrs::IfAddr;
use socket2::SockRef;

pub use self::reports::Reports;
use super::{Workspace, workspace::Universe};

type Universes = BTreeMap<u16, Universe>;

pub struct Lights {
    sent: Universes,
    sender: mpsc::Sender<Universes>,
    /// Why no light gets out, in words, until a round gets some out
    problem: Option<String>,
}

impl Lights {
    /// Starts the thread and returns the output with the reports of the
    /// thread, which the window subscribes to
    pub fn open() -> (Self, Reports) {
        let (sender, levels) = mpsc::channel();
        let (reporter, reports) = Reports::open();
        thread::spawn(move || sending::send(&levels, &reporter));
        let lights = Self {
            sent: Universes::new(),
            sender,
            problem: None,
        };
        (lights, reports)
    }

    pub fn sync(&mut self, workspace: &Workspace) {
        let universes = workspace.universes();
        if universes != self.sent {
            // Thread has gone only where it could not open a socket
            let _ = self.sender.send(universes.clone());
            self.sent = universes;
        }
    }

    pub fn told(&mut self, problem: Option<String>) {
        self.problem = problem;
    }

    /// Returns why no light gets out, while the patch has an instrument,
    /// since a show without one sends nothing
    pub fn problem(&self) -> Option<&str> {
        self.problem.as_deref().filter(|_| !self.sent.is_empty())
    }
}

/// Network interface of the computer by its IPv4 address.  The loopback
/// interface reaches programs on the computer itself, such as visualizers,
/// whose firewall can keep them from hearing broadcasts and multicast.
struct Interface {
    address: Ipv4Addr,
    broadcast: Option<Ipv4Addr>,
    loopback: bool,
}

impl Interface {
    /// Sends the packets of a universe and returns how each send went
    fn send(
        &self,
        socket: &UdpSocket,
        universe: u16,
        artnet: Option<&[u8]>,
        sacn: &[u8],
    ) -> Vec<io::Result<usize>> {
        let (artnet_to, sacn_to) = if self.loopback {
            let to = SocketAddr::from((self.address, sacn::PORT));
            (Some(self.address), Some(to))
        } else {
            (self.broadcast, sacn::group(universe))
        };
        // Interface gone since, which the next refresh leaves out
        let _ = SockRef::from(socket).set_multicast_if_v4(&self.address);
        let sacn_sent = sacn_to.map(|to| socket.send_to(sacn, to));
        let artnet_sent = artnet
            .zip(artnet_to)
            .map(|(packet, to)| socket.send_to(packet, (to, artnet::PORT)));
        sacn_sent.into_iter().chain(artnet_sent).collect()
    }
}

fn interfaces() -> Vec<Interface> {
    if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|interface| match interface.addr {
            IfAddr::V4(ref address) => Some(Interface {
                address: address.ip,
                broadcast: address.broadcast,
                loopback: interface.is_loopback(),
            }),
            IfAddr::V6(_) => None,
        })
        .collect()
}
