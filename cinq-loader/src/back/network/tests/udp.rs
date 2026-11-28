// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of UDP messages, to host names and to broadcast addresses

use super::*;

#[test]
fn udp_messages_go_out_as_one_datagram_each_also_to_a_host_name() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket.set_read_timeout(Some(PATIENCE)).unwrap();
    let port = socket.local_addr().unwrap().port();
    let network = Network::open();
    [
        (udp_to(&socket), 3),
        // Name that resolves to the IPv4 address
        (Destination::Udp(format!("localhost:{port}")), 4),
    ]
    .into_iter()
    .for_each(|(destination, byte)| {
        network.send(message(destination, byte));
        let mut datagram = [0; 16];
        let length = socket.recv(&mut datagram).unwrap();
        assert_eq!(datagram[..length], [byte; 4]);
    });
}

#[test]
fn a_host_that_cannot_be_found_is_told_in_few_words() {
    let error = ipv4_address("no-such-host.invalid:9000").unwrap_err();
    assert_eq!(error.to_string(), "cannot find the host");
    let error = ipv4_address("no port").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidInput);
}

#[test]
fn udp_messages_may_go_to_a_broadcast_address() {
    assert!(udp_socket().unwrap().broadcast().unwrap());
}

#[test]
fn a_burst_to_a_host_name_resolves_it_once() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket.set_read_timeout(Some(PATIENCE)).unwrap();
    let to = socket.local_addr().unwrap();
    let lookups = Cell::new(0);
    let resolve = |_: &str| {
        lookups.set(lookups.get() + 1);
        Ok(to)
    };
    let destination = Destination::Udp("console.local:9000".to_owned());
    (0..3).fold(None, |kept, byte| {
        let outgoing = message(destination.clone(), byte).remove(0);
        let kept = deliver(kept, outgoing, Instant::now(), resolve).unwrap();
        let mut datagram = [0; 16];
        let length = socket.recv(&mut datagram).unwrap();
        assert_eq!(datagram[..length], [byte; 4]);
        kept
    });
    assert_eq!(lookups.get(), 1);
}

#[test]
fn a_message_on_its_way_goes_out_before_the_network_is_dropped() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let network = Network::open();
    network.send(message(udp_to(&socket), 9));
    drop(network);
    // Nothing waits for the datagram now, so it has come or it never will
    socket.set_nonblocking(true).unwrap();
    let mut datagram = [0; 16];
    let length = socket.recv(&mut datagram).unwrap();
    assert_eq!(datagram[..length], [9; 4]);
}
