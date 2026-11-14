// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Network output, where the back end has side effects
//!
//! [`Network::send`] hands the messages that network cues sent in an update
//! to a thread of its own, which sends each to its destination: over a TCP
//! connection that it keeps open, or as one UDP datagram.  Each connection
//! has a thread that reads and drops whatever the destination sends back,
//! such as the state that a console reports, since a destination whose
//! replies go unread can stall, and that marks the connection closed once
//! the destination closes it, so that the next message opens a new one
//! rather than going into a connection that has gone.  The reason that a
//! message to a destination last failed stays, for the interface to show,
//! until a message gets there, and warns once on standard error.

#[cfg(test)]
mod tests;

use std::{
    collections::BTreeMap,
    io::{self, ErrorKind, Write},
    net::{Ipv4Addr, SocketAddr, TcpStream, ToSocketAddrs, UdpSocket},
    sync::{
        Arc, Mutex, MutexGuard, PoisonError,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use socket2::{SockRef, TcpKeepalive};

use super::osc::{self, Destination, Outgoing};
use crate::warn;

/// Longest wait for a destination to take a TCP connection or a message,
/// which holds back the messages after it, so that a console that is off
/// delays them by no more than this
const PATIENCE: Duration = Duration::from_secs(1);

pub struct Network {
    sender: mpsc::Sender<Vec<Outgoing>>,
    /// Why the last message to each destination that failed did, by
    /// destination, until a later one gets there
    problems: Arc<Mutex<BTreeMap<String, String>>>,
    /// Messages handed to the thread that it has yet to send
    pending: Arc<AtomicUsize>,
}

impl Network {
    pub fn open() -> Self {
        let (sender, messages) = mpsc::channel();
        let network = Self {
            sender,
            problems: Arc::default(),
            pending: Arc::default(),
        };
        let (problems, pending) = (Arc::clone(&network.problems), Arc::clone(&network.pending));
        thread::spawn(move || send(&messages, &problems, &pending));
        network
    }

    pub fn send(&self, messages: Vec<Outgoing>) {
        if !messages.is_empty() {
            self.pending.fetch_add(messages.len(), Ordering::Relaxed);
            // Thread has gone only where it could not open a socket
            let _ = self.sender.send(messages);
        }
    }

    /// Returns why messages to destinations failed, one line for each
    pub fn problems(&self) -> Vec<String> {
        locked(&self.problems)
            .iter()
            .map(|(destination, reason)| format!("Network cue to {destination}: {reason}"))
            .collect()
    }

    /// Returns whether messages are on their way, which can change the
    /// problems once they arrive or fail
    pub fn sending(&self) -> bool {
        self.pending.load(Ordering::Relaxed) > 0
    }
}

/// Locks the problems, which a thread that panicked while it held the
/// lock leaves as they were
fn locked(problems: &Mutex<BTreeMap<String, String>>) -> MutexGuard<'_, BTreeMap<String, String>> {
    problems.lock().unwrap_or_else(PoisonError::into_inner)
}

/// TCP connection to a destination, with whether the destination has
/// closed it
struct Connection {
    stream: TcpStream,
    closed: Arc<AtomicBool>,
}

fn send(
    messages: &mpsc::Receiver<Vec<Outgoing>>,
    problems: &Mutex<BTreeMap<String, String>>,
    pending: &AtomicUsize,
) {
    let Ok(udp) = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)) else {
        return;
    };
    let mut connections = BTreeMap::<String, Connection>::new();
    for Outgoing {
        destination,
        packet,
    } in messages.iter().flatten()
    {
        let sent = match &destination {
            Destination::Udp(address) => {
                resolved(address).and_then(|found| udp.send_to(&packet, found).map(drop))
            }
            Destination::Tcp(address) => {
                let frame = osc::framed(&packet);
                let written = |connection: io::Result<Connection>| {
                    connection.and_then(|mut connection| {
                        connection.stream.write_all(&frame)?;
                        Ok(connection)
                    })
                };
                // Connection kept open that fails anyway, which a new one
                // replaces
                connections
                    .remove(address)
                    .filter(|connection| !connection.closed.load(Ordering::Relaxed))
                    .and_then(|connection| written(Ok(connection)).ok())
                    .map_or_else(|| written(connected(address)), Ok)
                    .map(|connection| {
                        connections.insert(address.clone(), connection);
                    })
            }
        };
        match sent {
            Ok(()) => locked(problems).remove(&destination.to_string()),
            Err(error) => {
                warn::once(&format!("network cue to {destination}: {error}"));
                locked(problems).insert(destination.to_string(), error.to_string())
            }
        };
        pending.fetch_sub(1, Ordering::Relaxed);
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

/// Returns a connection to the first address of `address`, whose writes
/// give up as late as connecting does, so that a console that stops reading
/// holds back the messages after it no longer than that.  Keepalive probes
/// it each second of quiet, so that a console that lost its power or its
/// cable shows as closed within seconds, rather than taking the next message
/// into a connection that has gone without a word.
fn connected(address: &str) -> io::Result<Connection> {
    let stream = TcpStream::connect_timeout(&resolved(address)?, PATIENCE)?;
    stream.set_nodelay(true)?;
    stream.set_write_timeout(Some(PATIENCE))?;
    let keepalive = TcpKeepalive::new()
        .with_time(PATIENCE)
        .with_interval(PATIENCE);
    SockRef::from(&stream).set_tcp_keepalive(&keepalive)?;
    let closed = Arc::new(AtomicBool::new(false));
    let (replies, closing) = (stream.try_clone()?, Arc::clone(&closed));
    thread::spawn(move || {
        let _ = io::copy(&mut &replies, &mut io::sink());
        closing.store(true, Ordering::Relaxed);
    });
    Ok(Connection { stream, closed })
}
