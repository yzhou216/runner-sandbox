// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of listening for OSC commands, on sockets of the loopback address

use std::{
    io,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    thread,
    time::Duration,
};

use iced::futures::{FutureExt, Stream, StreamExt};

use super::{Heard, MESSAGES, Remote, commands, listen, pauses, reason};
use crate::back::osc::{self, tests::bundle};

const LOOPBACK: Ipv4Addr = Ipv4Addr::LOCALHOST;

/// Returns an address on the loopback that nothing listens on
fn free() -> SocketAddr {
    UdpSocket::bind((LOOPBACK, 0))
        .unwrap()
        .local_addr()
        .unwrap()
}

/// Sends `datagram` to `address` until the stream gives something, since
/// the thread may not have bound the socket yet
fn heard_after(stream: impl Stream<Item = Heard>, address: SocketAddr, datagram: &[u8]) -> Heard {
    let sender = UdpSocket::bind((LOOPBACK, 0)).unwrap();
    let mut stream = Box::pin(stream);
    (0..100)
        .find_map(|_| {
            sender.send_to(datagram, address).unwrap();
            thread::sleep(Duration::from_millis(50));
            stream.next().now_or_never().flatten()
        })
        .expect("a command within five seconds")
}

#[test]
fn datagram_gives_the_command_it_holds() {
    let address = free();
    let go = osc::packet("/go").unwrap();
    assert_eq!(
        heard_after(listen(address), address, &go),
        Heard::Command(LOOPBACK.into(), Remote::Go)
    );
}

#[test]
fn address_that_another_socket_holds_fails_with_the_reason() {
    let taken = UdpSocket::bind((LOOPBACK, 0)).unwrap();
    let address = taken.local_addr().unwrap();
    let Heard::Failed(reason) = heard_after(listen(address), address, b"") else {
        panic!("a failure");
    };
    assert_eq!(
        reason,
        "osc_listen: cannot listen, address in use; retrying"
    );
}

#[test]
fn port_comes_free_once_the_stream_has_gone() {
    let address = free();
    let go = osc::packet("/go").unwrap();
    let stream = listen(address);
    heard_after(stream, address, &go);
    let rebound = (0..40).any(|_| {
        thread::sleep(Duration::from_millis(50));
        UdpSocket::bind(address).is_ok()
    });
    assert!(rebound);
}

#[test]
fn setting_that_moves_between_addresses_of_a_port_listens_without_a_failure() {
    let go = osc::packet("/go").unwrap();
    let address = free();
    let wildcard = SocketAddr::from((Ipv4Addr::UNSPECIFIED, address.port()));
    heard_after(listen(wildcard), address, &go);
    // The thread of the first setting still holds the port for a wait
    assert_eq!(
        heard_after(listen(address), address, &go),
        Heard::Command(LOOPBACK.into(), Remote::Go)
    );
}

#[test]
fn commands_of_a_datagram_come_in_order_without_releases_and_unknown_addresses() {
    let from = LOOPBACK.into();
    let message = |text: &str| osc::packet(text).unwrap();
    let go = message("/go");
    assert_eq!(commands(&go, from), [Heard::Command(from, Remote::Go)]);
    assert_eq!(commands(&message("/go 0"), from), []);
    assert_eq!(
        commands(&message("/lights/on"), from),
        [Heard::Ignored(format!(
            "OSC from {from}: unknown address \"/lights/on\""
        ))]
    );
    assert_eq!(
        commands(b"not OSC", from),
        [Heard::Ignored(format!(
            "OSC from {from}: not an OSC packet"
        ))]
    );
    assert_eq!(
        commands(&bundle(&[&go, &message("/panic")]), from),
        [
            Heard::Command(from, Remote::Go),
            Heard::Command(from, Remote::Panic)
        ]
    );
}

#[test]
fn bundle_gives_the_commands_of_its_first_messages_only() {
    let go = osc::packet("/go").unwrap();
    let bundle = bundle(&vec![go.as_slice(); MESSAGES + 10]);
    let heard = vec![Heard::Command(LOOPBACK.into(), Remote::Go); MESSAGES]
        .into_iter()
        .chain([Heard::Ignored(format!(
            "OSC from {LOOPBACK}: more than {MESSAGES} messages, the rest dropped"
        ))])
        .collect::<Vec<_>>();
    assert_eq!(commands(&bundle, LOOPBACK.into()), heard);
}

#[test]
fn bind_failure_without_a_plain_kind_gives_the_text_of_the_system() {
    assert_eq!(
        reason(&io::Error::from(io::ErrorKind::AddrInUse)),
        "address in use"
    );
    assert_eq!(reason(&io::Error::other("no route")), "no route");
}

#[test]
fn port_that_comes_free_later_is_bound_and_the_stream_says_so() {
    let taken = UdpSocket::bind((LOOPBACK, 0)).unwrap();
    let address = taken.local_addr().unwrap();
    let mut stream = Box::pin(listen(address));
    let mut next = || {
        (0..300)
            .find_map(|_| {
                thread::sleep(Duration::from_millis(50));
                stream.next().now_or_never().flatten()
            })
            .expect("something within fifteen seconds")
    };
    assert!(matches!(next(), Heard::Failed(_)));
    drop(taken);
    assert_eq!(next(), Heard::Listening);
}

#[test]
fn receive_that_fails_at_once_is_waited_out_unless_it_timed_out_or_was_refused() {
    let error = |kind| io::Error::from(kind);
    assert!(pauses(&error(io::ErrorKind::Other)));
    assert!(pauses(&error(io::ErrorKind::PermissionDenied)));
    assert!(!pauses(&error(io::ErrorKind::TimedOut)));
    assert!(!pauses(&error(io::ErrorKind::WouldBlock)));
    assert!(!pauses(&error(io::ErrorKind::ConnectionRefused)));
}

#[test]
fn release_of_a_button_that_sends_a_wide_zero_gives_no_command() {
    let from = LOOPBACK.into();
    let wide = |tags: &[u8; 4], number: [u8; 8]| [&b"/go\0"[..], tags, &number].concat();
    assert_eq!(commands(&wide(b",h\0\0", 0_i64.to_be_bytes()), from), []);
    assert_eq!(commands(&wide(b",d\0\0", 0_f64.to_be_bytes()), from), []);
    let pressed = commands(&wide(b",d\0\0", 1_f64.to_be_bytes()), from);
    assert_eq!(pressed, [Heard::Command(from, Remote::Go)]);
}
