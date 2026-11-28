// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! TCP connections to destinations of network cues
//!
//! A connection belongs to the worker of its destination and lasts only
//! while messages to that destination follow each other within
//! [`PATIENCE`].  The first message after a quiet second makes a new
//! connection, whose handshake shows that the destination is there at that
//! moment.  A connection kept open longer could have lost its cable
//! without a word, and TCP would hold a message written into it and deliver
//! it once the cable went back in, firing a cue late.

use std::{
    io::{self, ErrorKind, Read, Write},
    iter,
    net::TcpStream,
    time::Instant,
};

use super::{PATIENCE, ipv4_address, time_left};

fn connected(address: &str, handed: Instant) -> io::Result<TcpStream> {
    let to = ipv4_address(address)?;
    let stream = TcpStream::connect_timeout(&to, time_left(handed)?)?;
    // Messages are small, and a cue must not wait for more to join them
    stream.set_nodelay(true)?;
    stream.set_write_timeout(Some(PATIENCE))?;
    Ok(stream)
}

/// Most reads of what a peer sent that one check makes, so that a peer that
/// sends faster than they drain it cannot hold the worker there
const READS: usize = 128;

/// Returns whether the destination still has `stream` open, after reading
/// away what it sent back, such as the state that a console reports, so that
/// it cannot stall on replies that nobody reads, and so that a close shows
/// as the end of the stream.  A peer that still has more than [`READS`] reads
/// to send counts as open, and the next check reads on.
fn still_open(stream: &TcpStream) -> bool {
    stream.set_nonblocking(true).is_ok()
        && iter::repeat_with(|| (&*stream).read(&mut [0; 512]))
            .take(READS)
            .find(|read| !matches!(read, Ok(length) if *length > 0))
            .is_none_or(|read| read.is_err_and(|error| error.kind() == ErrorKind::WouldBlock))
        && stream.set_nonblocking(false).is_ok()
}

/// Writes `frame` to the `kept` connection while the destination has it
/// open, or else to a new one, and returns it
pub(super) fn written(
    kept: Option<TcpStream>,
    address: &str,
    frame: &[u8],
    handed: Instant,
) -> io::Result<TcpStream> {
    kept.filter(still_open)
        .map_or_else(|| connected(address, handed), Ok)
        .and_then(|stream| {
            time_left(handed)?;
            (&stream)
                .write_all(frame)
                .map(|()| stream)
                .map_err(timed_out)
        })
}

/// Returns `error` as a timeout if it is the one that a write gives on
/// POSIX systems when its time runs out, which Windows already reports as one
fn timed_out(error: io::Error) -> io::Error {
    match error.kind() {
        ErrorKind::WouldBlock => ErrorKind::TimedOut.into(),
        _ => error,
    }
}
