// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! TCP connections to destinations of network cues
//!
//! Each connection has a thread that reads and drops whatever the
//! destination sends back, such as the state that a console reports, since
//! a destination whose replies go unread can stall.  The thread marks the
//! connection closed once the destination closes it, so that the next
//! message opens a new one rather than going into a connection that has
//! gone, and tells why when the connection was lost to an error.

use std::{
    io::{self, Write},
    net::{Shutdown, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use socket2::{SockRef, TcpKeepalive};

use super::{PATIENCE, Shared, resolved};
use crate::back::Destination;

pub(super) struct Connection {
    stream: TcpStream,
    closed: Arc<AtomicBool>,
}

impl Connection {
    /// Returns a connection to the first address of `address`, whose writes
    /// give up as late as connecting does, so that a console that stops
    /// reading holds back the messages after it no longer than that.
    /// Keepalive probes it each second of quiet, so that a console that
    /// lost its power or its cable shows as lost within seconds, among the
    /// problems of `destination`, rather than taking the next message into
    /// a connection that has gone without a word.
    pub(super) fn opened(
        address: &str,
        destination: &Destination,
        shared: &Arc<Shared>,
    ) -> io::Result<Self> {
        let stream = TcpStream::connect_timeout(&resolved(address)?, PATIENCE)?;
        stream.set_nodelay(true)?;
        stream.set_write_timeout(Some(PATIENCE))?;
        let keepalive = TcpKeepalive::new()
            .with_time(PATIENCE)
            .with_interval(PATIENCE);
        SockRef::from(&stream).set_tcp_keepalive(&keepalive)?;
        let replies = stream.try_clone()?;
        let closed = Arc::new(AtomicBool::new(false));
        let (closing, shared, destination) = (
            Arc::clone(&closed),
            Arc::clone(shared),
            destination.to_string(),
        );
        shared.open.fetch_add(1, Ordering::Relaxed);
        thread::spawn(move || {
            if let Err(error) = io::copy(&mut &replies, &mut io::sink()) {
                shared
                    .problems()
                    .insert(destination, format!("lost the connection: {error}"));
            }
            closing.store(true, Ordering::Relaxed);
            shared.open.fetch_sub(1, Ordering::Relaxed);
        });
        Ok(Self { stream, closed })
    }

    pub(super) fn is_open(&self) -> bool {
        !self.closed.load(Ordering::Relaxed)
    }

    /// Returns the connection once it has written `frame`, or else shuts it
    /// down, which ends the thread that reads from it, and tells why
    pub(super) fn written(mut self, frame: &[u8]) -> io::Result<Self> {
        match self.stream.write_all(frame) {
            Ok(()) => Ok(self),
            Err(error) => {
                let _ = self.stream.shutdown(Shutdown::Both);
                Err(error)
            }
        }
    }
}
