// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of messages going out over loopback

mod failures;
mod tcp;
mod udp;

use std::{
    cell::Cell,
    collections::BTreeMap,
    io::{ErrorKind, Read, Write},
    iter,
    net::{TcpListener, TcpStream, UdpSocket},
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

use super::{Network, address::unfound, connection, deliver, ipv4_address, udp_socket};
use crate::back::osc::{Destination, Outgoing, framed};

/// Longest that a test waits, which only a broken test reaches
const PATIENCE: Duration = Duration::from_secs(10);

fn tcp_to(listener: &TcpListener) -> Destination {
    Destination::Tcp(listener.local_addr().unwrap().to_string())
}

fn udp_to(socket: &UdpSocket) -> Destination {
    Destination::Udp(socket.local_addr().unwrap().to_string())
}

fn message(destination: Destination, byte: u8) -> Vec<Outgoing> {
    vec![Outgoing {
        destination,
        packet: vec![byte; 4],
    }]
}

fn accepted(listener: &TcpListener) -> TcpStream {
    listener.set_nonblocking(true).unwrap();
    let start = Instant::now();
    let (stream, _) = iter::repeat_with(|| listener.accept())
        .find_map(|accepted| {
            assert!(start.elapsed() < PATIENCE, "no connection arrived");
            thread::sleep(Duration::from_millis(10));
            accepted.ok()
        })
        .unwrap();
    // Taken from a listener that does not block, which some systems pass on
    stream.set_nonblocking(false).unwrap();
    stream.set_read_timeout(Some(PATIENCE)).unwrap();
    stream
}

fn next(mut stream: &TcpStream) -> Vec<u8> {
    let mut frame = vec![0; framed(&[0; 4]).len()];
    stream.read_exact(&mut frame).unwrap();
    frame
}

/// Returns the next frame on a connection of its own
fn frame(listener: &TcpListener) -> Vec<u8> {
    next(&accepted(listener))
}
