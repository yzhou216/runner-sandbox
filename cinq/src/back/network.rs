// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Network output, where the back end has side effects
//!
//! [`Network::send`] hands the messages that network cues sent in an update
//! to a thread of its own, which sends each to its destination: over a TCP
//! connection that it keeps open, or as one UDP datagram.  Each connection
//! has a thread that reads and drops whatever the destination sends back,
//! such as the state that Eos reports, since a destination whose replies
//! go unread can stall, and that marks the connection closed once the
//! destination closes it, so that the next message opens a new one rather
//! than going into a connection that has gone.  A message that cannot go
//! out warns once on standard error.

#[cfg(test)]
mod tests;

use std::{
    collections::BTreeMap,
    io::{self, ErrorKind, Write},
    net::{Ipv4Addr, TcpStream, ToSocketAddrs, UdpSocket},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use super::osc::{self, Destination, Outgoing};
use crate::warn;

/// Longest wait for a destination to take a TCP connection, which holds
/// back the messages after it, so that a console that is off delays them
/// by no more than this
const PATIENCE: Duration = Duration::from_secs(1);

pub struct Network {
    sender: mpsc::Sender<Vec<Outgoing>>,
}

impl Network {
    pub fn open() -> Self {
        let (sender, messages) = mpsc::channel();
        thread::spawn(move || send(&messages));
        Self { sender }
    }

    pub fn send(&self, messages: Vec<Outgoing>) {
        if !messages.is_empty() {
            // Thread has gone only where it could not open a socket
            let _ = self.sender.send(messages);
        }
    }
}

/// TCP connection to a destination, with whether the destination has
/// closed it
struct Connection {
    stream: TcpStream,
    closed: Arc<AtomicBool>,
}

fn send(messages: &mpsc::Receiver<Vec<Outgoing>>) {
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
            Destination::Udp(address) => udp.send_to(&packet, address.as_str()).map(drop),
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
        if let Err(error) = sent {
            warn::once(&format!("network cue to {destination}: {error}"));
        }
    }
}

fn connected(address: &str) -> io::Result<Connection> {
    let socket = address
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "no address for the host"))?;
    let stream = TcpStream::connect_timeout(&socket, PATIENCE)?;
    stream.set_nodelay(true)?;
    let closed = Arc::new(AtomicBool::new(false));
    let (replies, closing) = (stream.try_clone()?, Arc::clone(&closed));
    thread::spawn(move || {
        let _ = io::copy(&mut &replies, &mut io::sink());
        closing.store(true, Ordering::Relaxed);
    });
    Ok(Connection { stream, closed })
}
