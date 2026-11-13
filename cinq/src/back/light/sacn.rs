// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Packets of sACN, ANSI E1.31, which ETC's consoles and gateways speak
//! natively

use std::net::{Ipv4Addr, SocketAddrV4};

use super::Universe;

pub(super) const PORT: u16 = 5568;

/// Priority of a source that sets none, which Eos sends too, so that a
/// receiver merges cinq's levels with a console's, the highest of each
/// address winning, rather than leaving out the source of lower priority
const PRIORITY: u8 = 100;

/// Bytes from the start of a data packet to its first DMX slot
const HEADER: usize = 126;

/// Returns the identity of a source, a random UUID as E1.31 asks, which
/// tells receivers apart one cinq from another
pub(super) fn cid() -> [u8; 16] {
    let bytes = fastrand::u128(..).to_be_bytes();
    // Version 4 and the variant of RFC 4122
    std::array::from_fn(|i| match i {
        6 => bytes[i] & 0x0f | 0x40,
        8 => bytes[i] & 0x3f | 0x80,
        _ => bytes[i],
    })
}

/// Returns the multicast group of `universe`, 239.255 followed by the two
/// bytes of its number
pub(super) fn group(universe: u16) -> SocketAddrV4 {
    let [high, low] = universe.to_be_bytes();
    SocketAddrV4::new(Ipv4Addr::new(239, 255, high, low), PORT)
}

/// Returns the data packet of `universe` from the source `cid`, the
/// `sequence`th that the source has sent of it
pub(super) fn packet(cid: [u8; 16], universe: u16, sequence: u8, channels: &Universe) -> Vec<u8> {
    let end = HEADER + channels.len();
    // Each layer starts with its length to the end of the packet, under the
    // flags 0x7
    let length = |from: usize| (0x7000 | (end - from) as u16).to_be_bytes();
    let name: [u8; 64] = std::array::from_fn(|i| b"cinq".get(i).copied().unwrap_or(0));
    let slots = (channels.len() as u16 + 1).to_be_bytes();
    [
        &[0x00, 0x10, 0x00, 0x00][..],
        b"ASC-E1.17\0\0\0",
        &length(16),
        &4_u32.to_be_bytes(),
        &cid,
        &length(38),
        &2_u32.to_be_bytes(),
        &name,
        // Priority, no synchronization universe, sequence, and no options
        &[PRIORITY, 0, 0, sequence, 0],
        &universe.to_be_bytes(),
        &length(115),
        // Set property, its address and data types, first address 0, and
        // an increment of 1
        &[0x02, 0xa1, 0x00, 0x00, 0x00, 0x01],
        &slots,
        // Start code of levels
        &[0x00],
        channels,
    ]
    .concat()
}
