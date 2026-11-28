// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of reading the packets that arrive

use super::{Argument, Message, NESTING, packet};
use crate::back::osc::{self, tests::bundle};

fn message(address: &str, arguments: Vec<Argument>) -> Message {
    Message {
        address: address.to_owned(),
        arguments,
    }
}

#[test]
fn message_reads_as_its_address_and_arguments() {
    let bytes = osc::packet(r#"/cue/1/start 7 2.5 "Act 1""#).unwrap();
    assert_eq!(
        packet(&bytes),
        Some(vec![message(
            "/cue/1/start",
            vec![Argument::Int(7), Argument::Float(2.5), Argument::Other]
        )])
    );
}

#[test]
fn address_and_type_tags_are_padded_to_four_bytes() {
    for address in ["/a", "/ab", "/abc", "/abcd", "/abcde"] {
        let bytes = osc::packet(address).unwrap();
        assert_eq!(packet(&bytes), Some(vec![message(address, vec![])]));
    }
}

#[test]
fn message_without_a_type_tag_string_has_no_arguments() {
    assert_eq!(
        packet(b"/go\0"),
        Some(vec![message("/go", vec![])]),
        "padded address alone"
    );
    assert_eq!(packet(b"/go"), None, "address that ends unpadded");
}

#[test]
fn type_tags_without_data_and_those_with_other_sizes_are_skipped_in_step() {
    let bytes = [
        &b"/x\0\0,TFNIhdtb\0\0\0"[..],
        &[0; 8],
        &[0; 8],
        &[0; 8],
        &3_i32.to_be_bytes(),
        b"abc\0",
    ]
    .concat();
    let read = packet(&bytes).unwrap();
    assert_eq!(
        read[0].arguments,
        [
            Argument::Bool(true),
            Argument::Bool(false),
            Argument::Other,
            Argument::Other,
            Argument::Int(0),
            Argument::Float(0.0),
            Argument::Other,
            Argument::Other,
        ]
    );
}

#[test]
fn zero_and_false_are_off_and_other_arguments_are_not() {
    assert!(Argument::Int(0).is_off());
    assert!(Argument::Float(0.0).is_off());
    assert!(Argument::Bool(false).is_off());
    assert!(!Argument::Int(1).is_off());
    assert!(!Argument::Float(0.5).is_off());
    assert!(!Argument::Bool(true).is_off());
    assert!(!Argument::Other.is_off());
}

#[test]
fn bundle_reads_as_the_messages_inside_it_in_order() {
    let go = osc::packet("/go").unwrap();
    let panic = osc::packet("/panic").unwrap();
    let inner = bundle(&[&panic]);
    let read = packet(&bundle(&[&go, &inner])).unwrap();
    assert_eq!(
        read,
        [message("/go", vec![]), message("/panic", vec![])],
        "nested bundles read too"
    );
    assert_eq!(packet(&bundle(&[])), Some(Vec::new()));
}

#[test]
fn bundles_nest_only_so_deep() {
    let deep = (0..=NESTING).fold(osc::packet("/go").unwrap(), |inner, _| bundle(&[&inner]));
    assert_eq!(packet(&deep), None);
    let allowed = (0..NESTING - 1).fold(osc::packet("/go").unwrap(), |inner, _| bundle(&[&inner]));
    assert_eq!(packet(&allowed).unwrap().len(), 1);
}

#[test]
fn malformed_packets_are_refused() {
    let go = osc::packet("/go 1").unwrap();
    let bundled = bundle(&[&go]);
    let refused = [
        &b""[..],
        b"\0\0\0\0",
        b"go\0\0",
        b"/go\0,i\0\0\0\0",
        b"/go\0,x\0\0",
        &go[..go.len() - 1],
        &bundled[..bundled.len() - 1],
        &bundled[..12],
        b"#bundle\0",
        b"/\xff\xfe\0",
    ];
    refused
        .iter()
        .for_each(|bytes| assert_eq!(packet(bytes), None, "{bytes:?}"));
    // Size that runs past the packet, and a negative size
    let long = [&b"#bundle\0"[..], &[0; 8], &100_i32.to_be_bytes(), &go].concat();
    let negative = [&b"#bundle\0"[..], &[0; 8], &(-4_i32).to_be_bytes(), &go].concat();
    assert_eq!(packet(&long), None);
    assert_eq!(packet(&negative), None);
}
