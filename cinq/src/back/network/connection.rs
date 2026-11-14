// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! TCP connections to destinations of network cues
//!
//! A connection lasts only while messages to its destination follow each
//! other within [`PATIENCE`].  The first message after a quiet second makes
//! a new connection, whose handshake shows that the destination is there at
//! that moment.  A connection kept open longer could have lost its cable
//! without a word, and TCP would hold a message written into it and deliver
//! it once the cable went back in, firing a cue late.

use std::{
    io::{self, ErrorKind, Read, Write},
    iter,
    net::TcpStream,
    time::Instant,
};

use super::{PATIENCE, left, resolved};

/// Returns a connection to the first IPv4 address of `address`, made in
/// the time that a message handed over at `handed` has left, whose writes
/// give up within [`PATIENCE`]
fn connected(address: &str, handed: Instant) -> io::Result<TcpStream> {
    let to = resolved(address)?;
    let stream = TcpStream::connect_timeout(&to, left(handed)?)?;
    stream.set_nodelay(true)?;
    stream.set_write_timeout(Some(PATIENCE))?;
    Ok(stream)
}

/// Returns whether the destination still has `stream` open, once it has
/// read away whatever the destination sent back, such as the state that a
/// console reports
fn open(stream: &TcpStream) -> bool {
    stream.set_nonblocking(true).is_ok()
        && iter::repeat_with(|| (&*stream).read(&mut [0; 512]))
            .find(|read| !matches!(read, Ok(length) if *length > 0))
            .is_some_and(|read| read.is_err_and(|error| error.kind() == ErrorKind::WouldBlock))
        && stream.set_nonblocking(false).is_ok()
}

/// Returns the connection to `address` once it has written `frame`, the
/// message handed over at `handed`, if it still has time: `kept`, while the
/// destination has it open, or else a new one
pub(super) fn written(
    kept: Option<TcpStream>,
    address: &str,
    frame: &[u8],
    handed: Instant,
) -> io::Result<TcpStream> {
    kept.filter(open)
        .map_or_else(|| connected(address, handed), Ok)
        .and_then(|stream| {
            left(handed)?;
            (&stream).write_all(frame).map(|()| stream)
        })
}
