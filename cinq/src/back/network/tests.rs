// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of messages going out over loopback

use std::{
    io::Read,
    net::{TcpListener, UdpSocket},
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

use socket2::SockRef;

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
fn messages_to_a_host_name_go_to_its_ipv4_address() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket.set_read_timeout(Some(PATIENCE)).unwrap();
    let port = socket.local_addr().unwrap().port();
    Network::open().send(message(Destination::Udp(format!("localhost:{port}")), 4));
    let mut datagram = [0; 16];
    let length = socket.recv(&mut datagram).unwrap();
    assert_eq!(datagram[..length], [4; 4]);
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

/// Waits until `network` has sent or given up on every message
fn settled(network: &Network) {
    let start = Instant::now();
    while network.shared.pending.load(Ordering::Relaxed) > 0 && start.elapsed() < PATIENCE {
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn failure_to_reach_a_destination_shows_until_a_message_gets_there() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    let network = Network::open();
    network.send(message(Destination::Tcp(address.to_string()), 5));
    settled(&network);
    let problems = network.problems();
    assert_eq!(problems.len(), 1);
    assert!(problems[0].starts_with(&format!("Network cue to {address}: ")));
    let listener = TcpListener::bind(address).unwrap();
    network.send(message(Destination::Tcp(address.to_string()), 6));
    assert_eq!(frame(&listener), framed(&[6; 4]));
    settled(&network);
    assert!(network.problems().is_empty());
}

#[test]
fn message_that_waited_past_its_second_is_dropped_rather_than_sent_late() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket
        .set_read_timeout(Some(Duration::from_millis(500)))
        .unwrap();
    let destination = Destination::Udp(socket.local_addr().unwrap().to_string());
    let network = Network::open();
    let handed = Instant::now().checked_sub(Duration::from_secs(2)).unwrap();
    network.send_since(message(destination, 7), handed);
    settled(&network);
    assert!(
        network.problems()[0].ends_with("dropped after waiting a second behind earlier messages")
    );
    assert!(socket.recv(&mut [0; 16]).is_err());
}

#[test]
fn connection_lost_to_an_error_shows_before_the_next_message() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let destination = Destination::Tcp(listener.local_addr().unwrap().to_string());
    let network = Network::open();
    network.send(message(destination, 8));
    let (mut stream, _) = listener.accept().unwrap();
    let mut frame = vec![0; framed(&[0; 4]).len()];
    stream.read_exact(&mut frame).unwrap();
    // Reset rather than close, as a console that went away and came back
    // answers a connection that it no longer knows
    SockRef::from(&stream)
        .set_linger(Some(Duration::ZERO))
        .unwrap();
    drop(stream);
    let start = Instant::now();
    while network.problems().is_empty() && start.elapsed() < PATIENCE {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(network.problems()[0].contains(": lost the connection: "));
}
