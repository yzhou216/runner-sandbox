// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Packets of sACN, the standard of ANSI E1.31 for DMX over networks, which
//! the sacn crate lays out

use std::net::SocketAddr;

use sacn::packet::{
    ACN_SDT_MULTICAST_PORT, AcnRootLayerProtocol, DISCOVERY_UNI_PER_PAGE, DataPacketDmpLayer,
    DataPacketFramingLayer, E131_DEFAULT_PRIORITY, E131RootLayer, E131RootLayerData,
    UniverseDiscoveryPacketFramingLayer, UniverseDiscoveryPacketUniverseDiscoveryLayer,
    universe_to_ipv4_multicast_addr,
};
pub(super) use sacn::packet::{
    E131_DISCOVERY_UNIVERSE as DISCOVERY, E131_UNIVERSE_DISCOVERY_INTERVAL as DISCOVERY_INTERVAL,
};
use uuid::Uuid;

use super::Universe;

pub(super) const PORT: u16 = ACN_SDT_MULTICAST_PORT;

const NAME: &str = "cinq";

/// Returns the identity of a source, a random UUID as E1.31 asks, which
/// tells receivers apart one cinq from another
pub(super) fn cid() -> Uuid {
    Uuid::new_v4()
}

/// Returns the multicast group of `universe`
pub(super) fn group(universe: u16) -> Option<SocketAddr> {
    universe_to_ipv4_multicast_addr(universe).ok()?.as_socket()
}

/// Returns the data packet of `universe` from the source `cid`, the
/// `sequence`th that the source has sent of it.  Its priority is the
/// default of E1.31, which most consoles send too, so that a receiver
/// merges cinq's levels with theirs, the highest of each address winning,
/// rather than leaving out the source of lower priority.
pub(super) fn packet(cid: Uuid, universe: u16, sequence: u8, channels: &Universe) -> Vec<u8> {
    // Start code of levels, then the levels
    let values: Vec<u8> = [0].into_iter().chain(channels.iter().copied()).collect();
    let framing = DataPacketFramingLayer {
        source_name: NAME.into(),
        priority: E131_DEFAULT_PRIORITY,
        synchronization_address: 0,
        sequence_number: sequence,
        preview_data: false,
        stream_terminated: false,
        force_synchronization: false,
        universe,
        data: DataPacketDmpLayer {
            property_values: values.into(),
        },
    };
    AcnRootLayerProtocol {
        pdu: E131RootLayer {
            cid,
            data: E131RootLayerData::DataPacket(framing),
        },
    }
    .pack_alloc()
    .expect("a universe of DMX levels fits a data packet")
}

/// Returns the pages of universe discovery from the source `cid`, which
/// list `universes`, in ascending order, for receivers and monitors to find
/// what the source sends without a universe to look in
pub(super) fn discovery(cid: Uuid, universes: &[u16]) -> Vec<Vec<u8>> {
    let pages = universes.chunks(DISCOVERY_UNI_PER_PAGE);
    let last_page = u8::try_from(pages.len().saturating_sub(1)).unwrap_or(u8::MAX);
    pages
        .zip(0..=u8::MAX)
        .map(|(universes, page)| {
            let framing = UniverseDiscoveryPacketFramingLayer {
                source_name: NAME.into(),
                data: UniverseDiscoveryPacketUniverseDiscoveryLayer {
                    page,
                    last_page,
                    universes: universes.into(),
                },
            };
            AcnRootLayerProtocol {
                pdu: E131RootLayer {
                    cid,
                    data: E131RootLayerData::UniverseDiscoveryPacket(framing),
                },
            }
            .pack_alloc()
            .expect("a page of universes fits a discovery packet")
        })
        .collect()
}
