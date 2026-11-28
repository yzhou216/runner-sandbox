// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Addresses and sockets of the destinations of network cues

use std::{
    io::{self, ErrorKind},
    net::{Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket},
};

/// Network cues go out over IPv4, as the networks of consoles run, while a
/// resolver can give IPv6 first, as `::1` for `localhost`.
pub(super) fn ipv4_address(address: &str) -> io::Result<SocketAddr> {
    address
        .to_socket_addrs()
        .map_err(unfound)?
        .find(SocketAddr::is_ipv4)
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "no IPv4 address for the host"))
}

/// Words a failed lookup as cinq's own, since the resolver's message, such
/// as `failed to lookup address information: Name or service not known`,
/// runs past the sidebar and adds nothing that the operator can act on,
/// while an address that cannot be read keeps its own
pub(super) fn unfound(error: io::Error) -> io::Error {
    match error.kind() {
        ErrorKind::InvalidInput => error,
        kind => io::Error::new(kind, "cannot find the host"),
    }
}

/// Returns a socket that may send to a broadcast address, such as
/// `255.255.255.255`, which the system otherwise refuses with a permission
/// error
pub(super) fn udp_socket() -> io::Result<UdpSocket> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
    socket.set_broadcast(true)?;
    Ok(socket)
}
