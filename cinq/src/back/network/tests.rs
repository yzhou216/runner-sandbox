// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of messages going out over loopback

use std::{
    io::Read,
    net::{TcpListener, UdpSocket},
    thread,
    time::Duration,
};

use super::Network;
use crate::back::osc::{Destination, Outgoing, framed};

/// Longest that a test waits for a message, which only a broken test takes
const PATIENCE: Duration = Duration::from_secs(10);

fn message(destination: Destination, byte: u8) -> Vec<Outgoing> {
    vec![Outgoing {
        destination,
        packet: vec![byte; 4],
    }]
}

/// Returns the next SLIP frame that `listener` receives on a connection
/// of its own, which the destination closes once it has read the frame
fn frame(listener: &TcpListener) -> Vec<u8> {
    let (mut stream, _) = listener.accept().unwrap();
    stream.set_read_timeout(Some(PATIENCE)).unwrap();
    let mut frame = vec![0; framed(&[0; 4]).len()];
    stream.read_exact(&mut frame).unwrap();
    frame
}

#[test]
fn tcp_messages_go_out_framed_and_open_a_new_connection_once_one_closes() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let destination = Destination::Tcp(listener.local_addr().unwrap().to_string());
    let network = Network::open();
    network.send(message(destination.clone(), 1));
    assert_eq!(frame(&listener), framed(&[1; 4]));
    // Time for the thread that reads replies to find the connection closed
    thread::sleep(Duration::from_secs(1));
    network.send(message(destination, 2));
    assert_eq!(frame(&listener), framed(&[2; 4]));
}

#[test]
fn udp_messages_go_out_as_one_datagram_each() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket.set_read_timeout(Some(PATIENCE)).unwrap();
    let destination = Destination::Udp(socket.local_addr().unwrap().to_string());
    Network::open().send(message(destination, 3));
    let mut datagram = [0; 16];
    let length = socket.recv(&mut datagram).unwrap();
    assert_eq!(datagram[..length], [3; 4]);
}
