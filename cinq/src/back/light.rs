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
//! each of them, once it changes and again each second, as both protocols
//! ask of a source, so that a node that starts late or loses a packet
//! catches up.  A show without instruments sends nothing.
//!
//! When cinq ends, sACN receivers hold the levels for as long as they hold
//! those of a source that has gone quiet, since a stream that said it had
//! ended would let them drop the lights at once.

mod artnet;
mod sacn;
#[cfg(test)]
mod tests;

use std::{
    collections::BTreeMap,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    sync::mpsc::{self, RecvTimeoutError},
    thread,
    time::Duration,
};

use if_addrs::IfAddr;
use socket2::SockRef;

use super::{Workspace, workspace::Universe};

const REFRESH: Duration = Duration::from_secs(1);

type Universes = BTreeMap<u16, Universe>;

pub struct Lights {
    sent: Universes,
    sender: mpsc::Sender<Universes>,
}

impl Lights {
    pub fn open() -> Self {
        let (sender, levels) = mpsc::channel();
        thread::spawn(move || send(&levels));
        Self {
            sent: Universes::new(),
            sender,
        }
    }

    pub fn sync(&mut self, workspace: &Workspace) {
        let universes = workspace.universes();
        if universes != self.sent {
            // Thread has gone only where it could not open a socket
            let _ = self.sender.send(universes.clone());
            self.sent = universes;
        }
    }
}

/// Network that the computer is on, by the computer's own IPv4 address on
/// it, or the computer itself, by its loopback address, since a firewall
/// can keep a program on the same computer, such as a visualizer, from
/// hearing broadcasts and multicast
struct Network {
    address: Ipv4Addr,
    broadcast: Option<Ipv4Addr>,
    loopback: bool,
}

impl Network {
    /// Sends `artnet` and `sacn`, the packets of `universe`, to everyone on
    /// the network, or straight to the computer itself
    fn send(&self, socket: &UdpSocket, universe: u16, artnet: Option<&[u8]>, sacn: &[u8]) {
        let (artnet_to, sacn_to) = if self.loopback {
            let to = SocketAddr::from((self.address, sacn::PORT));
            (Some(self.address), Some(to))
        } else {
            (self.broadcast, sacn::group(universe))
        };
        // Network that has gone since, which the next refresh leaves out
        let _ = SockRef::from(socket).set_multicast_if_v4(&self.address);
        if let Some(to) = sacn_to {
            let _ = socket.send_to(sacn, to);
        }
        if let (Some(packet), Some(to)) = (artnet, artnet_to) {
            let _ = socket.send_to(packet, (to, artnet::PORT));
        }
    }
}

/// Sends the universes as they arrive, and the latest again each time that
/// none arrive for a while, until the other end goes
fn send(levels: &mpsc::Receiver<Universes>) {
    let Ok(socket) = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
        .and_then(|socket| socket.set_broadcast(true).map(|()| socket))
    else {
        return;
    };
    let cid = sacn::cid();
    let mut latest = Universes::new();
    // Each round sends every universe once, so that one count numbers the
    // packets of each in order, as sACN asks
    for sequence in (0..=u8::MAX).cycle() {
        latest = match levels.recv_timeout(REFRESH) {
            Ok(universes) => universes,
            Err(RecvTimeoutError::Timeout) => latest,
            Err(RecvTimeoutError::Disconnected) => return,
        };
        let networks = networks();
        latest.iter().for_each(|(&universe, channels)| {
            let artnet = artnet::packet(universe, channels);
            let sacn = sacn::packet(cid, universe, sequence, channels);
            networks
                .iter()
                .for_each(|network| network.send(&socket, universe, artnet.as_deref(), &sacn));
        });
    }
}

fn networks() -> Vec<Network> {
    if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|interface| match interface.addr {
            IfAddr::V4(ref address) => Some(Network {
                address: address.ip,
                broadcast: address.broadcast,
                loopback: interface.is_loopback(),
            }),
            IfAddr::V6(_) => None,
        })
        .collect()
}
