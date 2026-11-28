// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the problems shown for destinations that cannot be reached

use super::*;

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
    let destination = tcp_to(&listener);
    drop(listener);
    let named = named(std::slice::from_ref(&destination));
    let network = Network::open();
    network.send(message(destination.clone(), 5));
    settled(&network);
    let problems = network.problems(&named);
    assert_eq!(problems.len(), 1);
    // Windows tries a closed port again before it refuses, for longer than
    // a message waits
    let words = if cfg!(windows) {
        "connection timed out"
    } else {
        "connection refused"
    };
    assert_eq!(
        problems[0].1,
        format!("message not sent: {words}"),
        "the words without the destination or the error number"
    );
    let listener = TcpListener::bind(address).unwrap();
    network.send(message(destination, 6));
    assert_eq!(frame(&listener), framed(&[6; 4]));
    settled(&network);
    assert!(network.problems(&named).is_empty());
}

#[test]
fn message_that_waited_past_its_second_is_dropped_rather_than_sent_late() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket
        .set_read_timeout(Some(Duration::from_millis(500)))
        .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let destinations = [udp_to(&socket), tcp_to(&listener)];
    let network = Network::open();
    let handed = Instant::now().checked_sub(Duration::from_secs(2)).unwrap();
    network.send_since(
        destinations
            .iter()
            .cloned()
            .flat_map(|to| message(to, 7))
            .collect(),
        handed,
    );
    settled(&network);
    let problems = network.problems(&named(&destinations));
    assert_eq!(problems.len(), 2);
    assert!(problems.iter().all(|(_, problem)| {
        problem.ends_with(": dropped rather than sent more than a second late")
    }));
    assert!(socket.recv(&mut [0; 16]).is_err());
    listener.set_nonblocking(true).unwrap();
    assert!(
        listener
            .accept()
            .is_err_and(|error| error.kind() == ErrorKind::WouldBlock)
    );
}

fn named(destinations: &[Destination]) -> BTreeMap<String, ()> {
    destinations.iter().map(|to| (to.to_string(), ())).collect()
}

/// Returns a TCP destination that refuses every connection
fn refusing() -> Destination {
    tcp_to(&TcpListener::bind("127.0.0.1:0").unwrap())
}

#[test]
fn warning_of_a_destination_that_no_cue_names_any_more_goes_away() {
    let destination = refusing();
    let network = Network::open();
    network.send(message(destination.clone(), 8));
    settled(&network);
    let named = named(&[destination]);
    assert_eq!(network.problems(&named).len(), 1);
    assert!(network.problems(&BTreeMap::<String, ()>::new()).is_empty());
    assert!(network.problems(&named).is_empty());
}

#[test]
fn problem_comes_with_the_value_that_names_its_destination() {
    let destination = refusing();
    let network = Network::open();
    network.send(message(destination.clone(), 9));
    settled(&network);
    let named = BTreeMap::from([(destination.to_string(), 4_u8)]);
    let problems = network.problems(&named);
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].0, 4);
}
