// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the packets that go out

use std::{
    cell::Cell,
    collections::BTreeMap,
    io,
    time::{Duration, Instant},
};

use ::sacn::packet::{AcnRootLayerProtocol, E131RootLayerData};
use uuid::Uuid;

use super::{artnet, leaving::Leaving, listing::Listing, sacn, status::Round};

// Port-Address follows the 8-byte ID, opcode, protocol version, sequence,
// and physical port of an ArtDmx packet
fn port_address(packet: &[u8]) -> u16 {
    u16::from_le_bytes([packet[14], packet[15]])
}

#[test]
fn universes_go_out_as_the_port_address_before_them_as_consoles_send_them() {
    let channels = [0; 512];
    let address = |universe| artnet::packet(universe, &channels).map(|p| port_address(&p));
    assert_eq!(address(1), Some(0));
    assert_eq!(address(17), Some(16));
    assert_eq!(address(0), None);
}

#[test]
fn sacn_data_packet_holds_its_universe_levels_and_the_lengths_of_its_layers() {
    let channels = std::array::from_fn(|i| i as u8);
    let packet = sacn::packet(Uuid::from_bytes([7; 16]), 300, 9, &channels, false);
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

fn universes(numbers: &[u16]) -> BTreeMap<u16, [u8; 512]> {
    numbers.iter().map(|&number| (number, [255; 512])).collect()
}

#[test]
fn universe_that_leaves_goes_out_as_zeros_and_then_as_a_terminated_stream() {
    let mut leaving = Leaving::default().changed(&universes(&[1, 2]), &universes(&[2]));
    let frames: Vec<Vec<(u16, bool)>> = std::iter::from_fn(|| {
        (!leaving.is_empty()).then(|| {
            let frame = leaving.frame().collect();
            leaving = std::mem::take(&mut leaving).next();
            frame
        })
    })
    .collect();
    let terminated: Vec<bool> = frames.iter().map(|frame| frame[0].1).collect();
    assert_eq!(terminated, [false, false, false, true, true, true]);
    assert!(frames.iter().flatten().all(|&(universe, _)| universe == 1));
}

#[test]
fn universe_that_returns_is_no_longer_ended() {
    let leaving = Leaving::default()
        .changed(&universes(&[1]), &universes(&[]))
        .changed(&universes(&[]), &universes(&[1]));
    assert!(leaving.is_empty());
}

#[test]
fn closing_a_show_ends_every_universe_and_a_second_change_keeps_the_first_going() {
    let leaving = Leaving::default()
        .changed(&universes(&[1, 2]), &universes(&[]))
        .next()
        .changed(&universes(&[]), &universes(&[]));
    assert_eq!(leaving.frame().count(), 2);
}

#[test]
fn terminated_packet_sets_the_terminated_option_of_its_framing_layer() {
    let framing = |terminated| {
        let packet = sacn::packet(Uuid::nil(), 1, 0, &[0; 512], terminated);
        match AcnRootLayerProtocol::parse(&packet).unwrap().pdu.data {
            E131RootLayerData::DataPacket(framing) => framing.stream_terminated,
            other => panic!("{other:?} is not a data packet"),
        }
    };
    assert!(framing(true));
    assert!(!framing(false));
}

#[test]
fn interfaces_are_read_again_only_once_a_second() {
    let start = Instant::now();
    let reads = Cell::new(0);
    let read = || {
        reads.set(reads.get() + 1);
        vec![reads.get()]
    };
    let at = |seconds: f32| start + Duration::from_secs_f32(seconds);
    let listing = [0.0, 0.025, 0.5, 0.999]
        .into_iter()
        .fold(Listing::new(), |listing, seconds| {
            listing.at(at(seconds), read)
        });
    assert_eq!((reads.get(), listing.items()), (1, [1].as_slice()));
    let listing = listing.at(at(1.0), read).at(at(1.5), read);
    assert_eq!((reads.get(), listing.items()), (2, [2].as_slice()));
}

fn refused() -> io::Result<usize> {
    Err(io::ErrorKind::NetworkUnreachable.into())
}

#[test]
fn a_round_is_out_idle_or_stuck_in_words() {
    [
        // One packet out is out whatever the others do
        (Round::of(2, true, &[refused(), Ok(512)]), Round::Out),
        // Nothing due tells nothing
        (Round::of(0, false, &[]), Round::Idle),
        (Round::of(2, false, &[refused()]), Round::Idle),
        (
            Round::of(0, true, &[]),
            Round::Stuck("no network interface".to_owned()),
        ),
        (
            Round::of(1, true, &[refused(), refused()]),
            Round::Stuck("not sent: network unreachable".to_owned()),
        ),
    ]
    .into_iter()
    .for_each(|(round, wanted)| assert_eq!(round, wanted));
}

#[test]
fn a_problem_is_told_when_it_starts_and_when_it_ends_but_not_in_between() {
    let stuck = Round::Stuck("no network interface".to_owned());
    let problem = Some("no network interface".to_owned());
    assert_eq!(stuck.told(None), Some(problem.clone()));
    assert_eq!(stuck.told(problem.as_deref()), None);
    assert_eq!(Round::Out.told(problem.as_deref()), Some(None));
    assert_eq!(Round::Out.told(None), None);
    assert_eq!(Round::Idle.told(problem.as_deref()), None);
}
