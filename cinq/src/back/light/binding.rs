// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Opening the socket of the light thread, again while that fails
//!
//! A failure to bind can be a moment's, as when no descriptor is free, so
//! that the thread tries again instead of ending, as devices and OSC ports
//! do, and the show then sends without a restart.

#[cfg(test)]
mod tests;

use std::{
    io,
    net::{Ipv4Addr, UdpSocket},
    sync::mpsc::{self, RecvTimeoutError},
    time::{Duration, Instant},
};

use super::{Universes, reports::Reporter, sending::tell};
use crate::warn;

/// Time between tries to open the socket
pub(super) const RETRY: Duration = Duration::from_secs(2);

/// Opens the socket that every packet goes out on
pub(super) fn open() -> io::Result<UdpSocket> {
    UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
        .and_then(|socket| socket.set_broadcast(true).map(|()| socket))
}

/// Returns the socket that `bind` gives, trying again every `retry` while it
/// fails, with the levels that came meanwhile, of which the latest are the
/// ones to send.  The first failure is told, and so is the success after
/// it.  Returns none once the sender of the levels is dropped.
pub(super) fn bound(
    levels: &mpsc::Receiver<Universes>,
    reports: &Reporter,
    bind: impl Fn() -> io::Result<UdpSocket>,
    retry: Duration,
) -> Option<(UdpSocket, Universes)> {
    let mut latest = Universes::new();
    let mut failed = false;
    loop {
        match bind() {
            Ok(socket) => {
                if failed {
                    tell(reports, None);
                }
                return Some((socket, latest));
            }
            Err(error) => {
                if !failed {
                    tell(reports, Some(format!("not sent: {}", warn::words(&error))));
                    failed = true;
                }
                latest = waited(levels, latest, retry)?;
            }
        }
    }
}

/// Returns the latest levels that arrive within `retry`, or `latest` when
/// none do, taking every change of a fade as it comes, which must not make
/// the thread try again for each; returns none once the sender is dropped
fn waited(
    levels: &mpsc::Receiver<Universes>,
    mut latest: Universes,
    retry: Duration,
) -> Option<Universes> {
    let end = Instant::now() + retry;
    loop {
        match levels.recv_timeout(end.saturating_duration_since(Instant::now())) {
            Ok(universes) => latest = universes,
            Err(RecvTimeoutError::Timeout) => return Some(latest),
            Err(RecvTimeoutError::Disconnected) => return None,
        }
    }
}
