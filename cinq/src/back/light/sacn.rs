// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Packets of sACN, which is ANSI E1.31, laid out by the sacn crate

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

/// Returns the multicast address of `universe`, or none for one that sACN
/// has none for
pub(super) fn group(universe: u16) -> Option<SocketAddr> {
    universe_to_ipv4_multicast_addr(universe).ok()?.as_socket()
}

fn pack(cid: Uuid, data: E131RootLayerData) -> Vec<u8> {
    AcnRootLayerProtocol {
        pdu: E131RootLayer { cid, data },
    }
    .pack_alloc()
    .expect("a universe or a page of universes fits a packet")
}

/// Returns the data packet of `universe`.  Its priority is the E1.31
/// default, which most consoles send too, so that receivers merge cinq's
/// levels with theirs by the highest level of each address rather than
/// leaving out the source of lower priority.  A terminated packet tells
/// receivers that the source has ended the universe.
pub(super) fn packet(
    cid: Uuid,
    universe: u16,
    sequence: u8,
    channels: &Universe,
    terminated: bool,
) -> Vec<u8> {
    // Start code of levels, then the levels
    let values: Vec<u8> = [0].into_iter().chain(channels.iter().copied()).collect();
    pack(
        cid,
        E131RootLayerData::DataPacket(DataPacketFramingLayer {
            source_name: NAME.into(),
            priority: E131_DEFAULT_PRIORITY,
            synchronization_address: 0,
            sequence_number: sequence,
            preview_data: false,
            stream_terminated: terminated,
            force_synchronization: false,
            universe,
            data: DataPacketDmpLayer {
                property_values: values.into(),
            },
        }),
    )
}

/// Returns the discovery pages of `universes`, which must ascend, as E1.31
/// asks
pub(super) fn discovery(cid: Uuid, universes: &[u16]) -> Vec<Vec<u8>> {
    let pages = universes.chunks(DISCOVERY_UNI_PER_PAGE);
    let last_page = u8::try_from(pages.len().saturating_sub(1)).unwrap_or(u8::MAX);
    pages
        .zip(0..=u8::MAX)
        .map(|(universes, page)| {
            pack(
                cid,
                E131RootLayerData::UniverseDiscoveryPacket(UniverseDiscoveryPacketFramingLayer {
                    source_name: NAME.into(),
                    data: UniverseDiscoveryPacketUniverseDiscoveryLayer {
                        page,
                        last_page,
                        universes: universes.into(),
                    },
                }),
            )
        })
        .collect()
}
