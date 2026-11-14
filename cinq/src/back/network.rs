// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Network output, where the back end has side effects
//!
//! [`Network::send`] hands the messages that network cues sent in an update
//! to a thread of its own, which sends each to its destination: over a TCP
//! connection that it keeps open, as [`connection`] tells, or as one UDP
//! datagram.  A message that waited more than [`PATIENCE`] behind others,
//! as behind a console that does not answer, is dropped rather than sent
//! late, since a cue of a console that fires late, once the console is
//! back, does more harm than one that does not fire.  Why a message to a
//! destination last failed, or why its connection was lost, stays for the
//! interface to show until a message gets there, and warns once on standard
//! error.

mod connection;
#[cfg(test)]
mod tests;

use std::{
    collections::BTreeMap,
    io::{self, ErrorKind},
    net::{Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket},
    sync::{
        Arc, Mutex, MutexGuard, PoisonError,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

use super::osc::{self, Destination, Outgoing};
use crate::warn;
use connection::Connection;

/// Longest wait for a destination to take a TCP connection or a message,
/// and for a message to wait behind others
const PATIENCE: Duration = Duration::from_secs(1);

/// How often the interface looks for news of the output while messages are
/// on their way
const SENDING_CHECK: Duration = Duration::from_millis(100);

/// How often the interface looks for news of the output while connections
/// are open, so that it shows a console that has gone before the next cue
const OPEN_CHECK: Duration = Duration::from_secs(1);

/// What the thread that sends shares with the interface and with the
/// threads that read from connections
#[derive(Default)]
struct Shared {
    /// Why the last message to each destination failed, or why its
    /// connection was lost, by destination, until a later message gets there
    problems: Mutex<BTreeMap<String, String>>,
    /// Messages handed over that the thread has yet to send
    pending: AtomicUsize,
    /// TCP connections open
    open: AtomicUsize,
}

impl Shared {
    /// Locks the problems, which a thread that panicked while it held the
    /// lock leaves as they were
    fn problems(&self) -> MutexGuard<'_, BTreeMap<String, String>> {
        self.problems.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub struct Network {
    /// Messages, each with when it was handed over
    sender: mpsc::Sender<Vec<(Instant, Outgoing)>>,
    shared: Arc<Shared>,
}

impl Network {
    pub fn open() -> Self {
        let (sender, messages) = mpsc::channel();
        let shared = Arc::<Shared>::default();
        let sending = Arc::clone(&shared);
        thread::spawn(move || send(&messages, &sending));
        Self { sender, shared }
    }

    pub fn send(&self, messages: Vec<Outgoing>) {
        self.send_since(messages, Instant::now());
    }

    /// Hands over `messages` as though at `handed`
    fn send_since(&self, messages: Vec<Outgoing>, handed: Instant) {
        if !messages.is_empty() {
            self.shared
                .pending
                .fetch_add(messages.len(), Ordering::Relaxed);
            let stamped = messages
                .into_iter()
                .map(|message| (handed, message))
                .collect();
            // Thread has gone only where it could not open a socket
            let _ = self.sender.send(stamped);
        }
    }

    /// Returns why messages to destinations failed, one line for each
    pub fn problems(&self) -> Vec<String> {
        self.shared
            .problems()
            .iter()
            .map(|(destination, reason)| format!("Network cue to {destination}: {reason}"))
            .collect()
    }

    /// Returns how often the interface should look for news of the output,
    /// which can change the problems: often while messages are on their
    /// way, each second while connections are open, and never otherwise
    pub fn checks(&self) -> Option<Duration> {
        if self.shared.pending.load(Ordering::Relaxed) > 0 {
            Some(SENDING_CHECK)
        } else if self.shared.open.load(Ordering::Relaxed) > 0 {
            Some(OPEN_CHECK)
        } else {
            None
        }
    }
}

fn send(messages: &mpsc::Receiver<Vec<(Instant, Outgoing)>>, shared: &Arc<Shared>) {
    let Ok(udp) = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)) else {
        return;
    };
    let mut connections = BTreeMap::<String, Connection>::new();
    for (
        handed,
        Outgoing {
            destination,
            packet,
        },
    ) in messages.iter().flatten()
    {
        let sent = match &destination {
            _ if handed.elapsed() > PATIENCE => Err(io::Error::new(
                ErrorKind::TimedOut,
                "dropped after waiting a second behind earlier messages",
            )),
            Destination::Udp(address) => {
                resolved(address).and_then(|to| udp.send_to(&packet, to).map(drop))
            }
            Destination::Tcp(address) => {
                let frame = osc::framed(&packet);
                // Connection kept open that fails anyway, which a new one
                // replaces
                connections
                    .remove(address)
                    .filter(Connection::is_open)
                    .and_then(|connection| connection.written(&frame).ok())
                    .map_or_else(
                        || Connection::opened(address, &destination, shared)?.written(&frame),
                        Ok,
                    )
                    .map(|connection| {
                        connections.insert(address.clone(), connection);
                    })
            }
        };
        match sent {
            Ok(()) => shared.problems().remove(&destination.to_string()),
            Err(error) => {
                warn::once(&format!("network cue to {destination}: {error}"));
                shared
                    .problems()
                    .insert(destination.to_string(), error.to_string())
            }
        };
        shared.pending.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Returns the first IPv4 address that `address`, a host and a port, reads
/// as.  Network cues go out over IPv4, as the networks of consoles run,
/// while a resolver can give IPv6 first, as `::1` for `localhost`.
fn resolved(address: &str) -> io::Result<SocketAddr> {
    address
        .to_socket_addrs()?
        .find(SocketAddr::is_ipv4)
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "no IPv4 address for the host"))
}
