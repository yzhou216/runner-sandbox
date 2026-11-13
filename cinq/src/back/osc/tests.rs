// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how OSC reads and what goes out

use super::{Destination, framed, packet};

#[test]
fn message_without_arguments_holds_its_address_and_an_empty_type_tag() {
    assert_eq!(
        packet("/eos/cue/1/fire").unwrap(),
        b"/eos/cue/1/fire\0,\0\0\0"
    );
}

#[test]
fn arguments_read_as_integers_decimals_or_strings_and_quotes_keep_strings() {
    let expected = [
        &b"/x\0\0,ifss\0\0\0"[..],
        &7_i32.to_be_bytes(),
        &2.5_f32.to_be_bytes(),
        b"Chan 1 At Full#\0",
        b"12\0\0",
    ]
    .concat();
    assert_eq!(
        packet(r#" /x 7 2.5 "Chan 1 At Full#" "12" "#).unwrap(),
        expected
    );
    assert_eq!(packet("/x nan").unwrap()[4..], *b",s\0\0nan\0");
}

#[test]
fn message_starts_with_an_address() {
    assert_eq!(packet(""), None);
    assert_eq!(packet("eos/cue/1/fire"), None);
    assert_eq!(packet(r#""/eos/ping""#), None);
}

#[test]
fn slip_frames_escape_their_end_and_escape_bytes() {
    assert_eq!(
        framed(&[1, 0xc0, 0xdb]),
        [0xc0, 1, 0xdb, 0xdc, 0xdb, 0xdd, 0xc0]
    );
}

#[test]
fn destinations_read_as_host_and_port_over_tcp_or_udp() {
    let tcp = |address: &str| Some(Destination::Tcp(address.to_owned()));
    assert_eq!(
        Destination::parse(" 10.101.100.20:3037 "),
        tcp("10.101.100.20:3037")
    );
    assert_eq!(Destination::parse("tcp://ion:3037"), tcp("ion:3037"));
    assert_eq!(
        Destination::parse("udp://10.0.0.5:53000"),
        Some(Destination::Udp("10.0.0.5:53000".to_owned()))
    );
    assert_eq!(Destination::parse("10.101.100.20"), None);
    assert_eq!(Destination::parse(":3037"), None);
    assert_eq!(Destination::parse("ion:0"), None);
    assert_eq!(Destination::parse("ion:65536"), None);
}
