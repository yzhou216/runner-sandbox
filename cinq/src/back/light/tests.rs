// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the packets that go out

use std::net::Ipv4Addr;

use ::sacn::packet::{AcnRootLayerProtocol, E131RootLayerData};
use if_addrs::Ifv4Addr;
use uuid::Uuid;

use super::{artnet, broadcast, sacn};

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

#[test]
fn network_without_a_broadcast_address_takes_the_one_of_its_mask() {
    let address = |broadcast| Ifv4Addr {
        ip: Ipv4Addr::new(10, 101, 50, 50),
        netmask: Ipv4Addr::new(255, 255, 0, 0),
        prefixlen: 16,
        broadcast,
    };
    assert_eq!(broadcast(&address(None)), Ipv4Addr::new(10, 101, 255, 255));
    let reported = Ipv4Addr::new(10, 101, 0, 255);
    assert_eq!(broadcast(&address(Some(reported))), reported);
}

#[test]
fn universes_are_listed_for_discovery_on_pages_of_512() {
    let universes: Vec<u16> = (1..=600).collect();
    let pages = sacn::discovery(Uuid::nil(), &universes);
    let listed: Vec<(u8, u8, Vec<u16>)> = pages
        .iter()
        .map(
            |page| match AcnRootLayerProtocol::parse(page).unwrap().pdu.data {
                E131RootLayerData::UniverseDiscoveryPacket(framing) => (
                    framing.data.page,
                    framing.data.last_page,
                    framing.data.universes.into_owned(),
                ),
                other => panic!("{other:?} is not universe discovery"),
            },
        )
        .collect();
    assert_eq!(
        listed,
        [
            (0, 1, universes[..512].to_vec()),
            (1, 1, universes[512..].to_vec())
        ]
    );
    assert_eq!(
        sacn::group(sacn::DISCOVERY)
            .map(|group| group.to_string())
            .as_deref(),
        Some("239.255.250.214:5568")
    );
}
