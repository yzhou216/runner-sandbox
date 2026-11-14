// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the packets that go out

use uuid::Uuid;

use super::{artnet, sacn};

/// Port-Address of an ArtDmx packet, which follows its 8-byte ID, opcode,
/// protocol version, sequence, and physical port
fn port_address(packet: &[u8]) -> u16 {
    u16::from_le_bytes([packet[14], packet[15]])
}

#[test]
fn universes_go_out_as_the_port_address_before_them_as_consoles_send_them() {
    let channels = [0; 512];
    assert_eq!(
        artnet::packet(1, &channels).map(|p| port_address(&p)),
        Some(0)
    );
    assert_eq!(
        artnet::packet(17, &channels).map(|p| port_address(&p)),
        Some(16)
    );
    assert_eq!(artnet::packet(0, &channels), None);
}

#[test]
fn sacn_data_packet_holds_its_universe_levels_and_the_lengths_of_its_layers() {
    let channels = std::array::from_fn(|i| i as u8);
    let packet = sacn::packet(Uuid::from_bytes([7; 16]), 300, 9, &channels);
    let word = |at: usize| u16::from_be_bytes([packet[at], packet[at + 1]]);
    assert_eq!(packet.len(), 638);
    assert_eq!(&packet[4..16], b"ASC-E1.17\0\0\0");
    assert_eq!(
        [word(16), word(38), word(115)],
        [0x7000 | 622, 0x7000 | 600, 0x7000 | 523]
    );
    assert_eq!(&packet[44..48], b"cinq");
    assert_eq!([packet[108], packet[111]], [100, 9]);
    assert_eq!([word(113), word(123)], [300, 513]);
    assert_eq!(&packet[125..129], [0, 0, 1, 2]);
    assert_eq!(
        sacn::group(300).map(|group| group.to_string()).as_deref(),
        Some("239.255.1.44:5568")
    );
}
