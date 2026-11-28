// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how OSC reads and what goes out

use super::{Destination, framed, packet};

/// Returns a bundle of `elements`, each with its size, after a time tag
pub(super) fn bundle(elements: &[&[u8]]) -> Vec<u8> {
    let sized = elements
        .iter()
        .flat_map(|element| [&(element.len() as i32).to_be_bytes()[..], element].concat());
    [b"#bundle\0", &1_u64.to_be_bytes()[..]]
        .concat()
        .into_iter()
        .chain(sized)
        .collect()
}

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
fn message_holds_no_null_since_one_ends_a_string_of_osc() {
    assert_eq!(packet("/a\0b 1"), None);
    assert_eq!(packet("/a \"b\0c\""), None);
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

#[test]
fn hosts_hold_no_colon_bracket_or_slash_since_none_resolves() {
    assert_eq!(Destination::parse("[::1]:9000"), None);
    assert_eq!(Destination::parse("http://x:80"), None);
    assert_eq!(Destination::parse("udp://a:b:80"), None);
    assert_eq!(Destination::parse("tcp://tcp://ion:3037"), None);
    assert_eq!(Destination::parse("ion/x:3037"), None);
    assert!(Destination::parse("eos.local:3037").is_some());
}

#[test]
fn message_closes_every_double_quote_that_it_opens() {
    assert_eq!(packet("/scene \"Act 1"), None);
    assert_eq!(packet("/eos/newcmd \"Chan 5 At 50 Enter"), None);
    assert_eq!(packet("/scene Act\" 1"), None);
    assert!(packet("/scene \"Act 1\" \"2\"").is_some());
}

#[test]
fn numeric_hosts_are_ipv4_addresses_as_the_standard_library_reads_them() {
    // The system resolver reads a zero in front of an octet as octal and 0x
    // as hexadecimal, so that 010.0.0.1 would reach 8.0.0.1
    assert!(Destination::parse("10.101.100.20:3037").is_some());
    assert_eq!(Destination::parse("010.101.100.020:3037"), None);
    assert_eq!(Destination::parse("udp://192.168.001.100:9000"), None);
    assert_eq!(Destination::parse("127.1:9000"), None);
    assert_eq!(Destination::parse("1.2.3:80"), None);
    assert_eq!(Destination::parse("0x7f.0.0.1:9000"), None);
    assert_eq!(Destination::parse("0X7F000001:9000"), None);
    assert_eq!(Destination::parse("console.1:9000"), None);
    assert!(Destination::parse("10.0.0.1.example.com:80").is_some());
}

#[test]
fn host_with_a_null_is_not_a_destination() {
    assert_eq!(Destination::parse("con\0sole:3037"), None);
    assert_eq!(Destination::parse("udp://\0:9000"), None);
    assert!(!Destination::port_is_bad("con\0sole:0"));
}
