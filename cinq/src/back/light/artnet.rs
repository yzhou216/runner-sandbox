// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Packets of Art-Net, which nodes of every make understand

use artnet_protocol::{ArtCommand, Output};

use super::Universe;

pub(super) const PORT: u16 = 6454;

/// Returns the ArtDmx packet of `universe`, which goes out as the
/// Port-Address before it, since Art-Net counts from 0 where sACN counts
/// from 1
pub(super) fn packet(universe: u16, channels: &Universe) -> Option<Vec<u8>> {
    let output = Output {
        port_address: universe.checked_sub(1)?.try_into().ok()?,
        data: channels.to_vec().into(),
        ..Output::default()
    };
    ArtCommand::Output(output).write_to_buffer().ok()
}
