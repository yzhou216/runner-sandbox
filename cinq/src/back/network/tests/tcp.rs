// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of messages going out over TCP connections

use super::*;

#[test]
fn tcp_messages_of_a_burst_go_out_framed_over_one_connection() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let destination = tcp_to(&listener);
    let network = Network::open();
    network.send(message(destination.clone(), 1));
    network.send(message(destination, 2));
    let stream = accepted(&listener);
    assert_eq!(next(&stream), framed(&[1; 4]));
    assert_eq!(next(&stream), framed(&[2; 4]));
}

#[test]
fn tcp_message_after_a_quiet_second_goes_over_a_new_connection_while_others_go_out() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let destination = tcp_to(&listener);
    let elsewhere = UdpSocket::bind("127.0.0.1:0").unwrap();
    let other = udp_to(&elsewhere);
    let network = Network::open();
    network.send(message(destination.clone(), 1));
    let mut first = accepted(&listener);
    assert_eq!(next(&first), framed(&[1; 4]));
    // Five pauses of 300 ms outlast the quiet second
    (0..5).for_each(|_| {
        thread::sleep(Duration::from_millis(300));
        network.send(message(other.clone(), 2));
    });
    network.send(message(destination, 3));
    assert_eq!(frame(&listener), framed(&[3; 4]));
    assert_eq!(first.read(&mut [0; 16]).unwrap(), 0);
}

#[test]
fn tcp_message_after_the_destination_closed_goes_over_a_new_connection() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let destination = tcp_to(&listener);
    let network = Network::open();
    network.send(message(destination.clone(), 1));
    assert_eq!(frame(&listener), framed(&[1; 4]));
    // Time for the close to arrive, well within a burst
    thread::sleep(Duration::from_millis(100));
    network.send(message(destination, 2));
    assert_eq!(frame(&listener), framed(&[2; 4]));
}

/// Returns a listener whose backlog is full, so that a connection to it
/// hangs, with the connections that fill it.  Only Linux drops the
/// handshakes of such a listener, while other systems refuse them at once
/// or take them, so the tests that use it run on Linux only.
#[cfg(target_os = "linux")]
fn unresponsive() -> (socket2::Socket, Vec<TcpStream>) {
    let listener =
        socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::STREAM, None).unwrap();
    listener
        .bind(
            &"127.0.0.1:0"
                .parse::<std::net::SocketAddr>()
                .unwrap()
                .into(),
        )
        .unwrap();
    listener.listen(0).unwrap();
    let address = listener.local_addr().unwrap().as_socket().unwrap();
    let filled: Vec<TcpStream> =
        iter::repeat_with(|| TcpStream::connect_timeout(&address, Duration::from_millis(200)))
            .map_while(Result::ok)
            .take(16)
            .collect();
    assert!(filled.len() < 16, "the backlog did not fill");
    (listener, filled)
}

#[test]
#[cfg(target_os = "linux")]
fn tcp_destination_that_does_not_answer_does_not_delay_messages_to_others() {
    let (silent, _filled) = unresponsive();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket
        .set_read_timeout(Some(Duration::from_millis(500)))
        .unwrap();
    let batch = [
        Destination::Tcp(
            silent
                .local_addr()
                .unwrap()
                .as_socket()
                .unwrap()
                .to_string(),
        ),
        udp_to(&socket),
    ]
    .into_iter()
    .flat_map(|to| message(to, 8))
    .collect();
    let network = Network::open();
    network.send(message(udp_to(&socket), 9));
    assert!(
        socket.recv(&mut [0; 16]).is_ok(),
        "control: a datagram alone arrives"
    );
    network.send(batch);
    assert!(
        socket.recv(&mut [0; 16]).is_ok(),
        "the datagram was dropped as late"
    );
}

#[test]
#[cfg_attr(
    windows,
    ignore = "Windows takes in 64 MiB for a peer that never reads"
)]
fn write_that_times_out_is_told_as_timed_out() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap().to_string();
    // The peer accepts and never reads, so the buffers fill and the write waits
    let error =
        connection::written(None, &address, &vec![0; 64 << 20], Instant::now()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::TimedOut);
    drop(listener);
}

#[test]
fn check_of_a_connection_reads_a_bounded_amount_of_what_the_peer_sent() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let kept = TcpStream::connect(&address).unwrap();
    let kept_at = kept.local_addr().unwrap();
    let peer = accepted(&listener);
    // Fills every buffer on the way, as a peer that sends without pause does.
    // A write can find its buffer full before what it holds has moved on to
    // the other end, as on FreeBSD, so the peer writes again after a pause
    // until nothing more goes in.
    peer.set_nonblocking(true).unwrap();
    let sent = || {
        thread::sleep(Duration::from_millis(50));
        iter::repeat(&[0; 1 << 16])
            .map_while(|chunk| (&peer).write(chunk).ok())
            .sum::<usize>()
    };
    iter::repeat_with(sent)
        .take_while(|&sent| sent > 0)
        .for_each(drop);
    let stream = connection::written(Some(kept), &address, &[1; 4], Instant::now()).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap();
    assert!(
        stream.peek(&mut [0; 1]).is_ok(),
        "the check read everything, and would read on while the peer sends"
    );
    assert_eq!(
        stream.local_addr().unwrap(),
        kept_at,
        "a peer that is still open keeps its connection"
    );
}
