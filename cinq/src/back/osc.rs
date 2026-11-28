// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Open Sound Control, which network cues send, as text that the operator
//! writes and as the packets that go out
//!
//! A destination reads as `host:port` for TCP, with packets framed by SLIP
//! as OSC 1.1 asks of streams, or as `udp://host:port` for UDP, one packet
//! to a datagram.  A message reads as its address, then its arguments
//! after spaces, each an integer, a decimal number, or else a string,
//! which double quotes keep whole across spaces, as in
//! `/scene/recall "Act 1" 2`.
//!
//! The encoding, and the decoding of the packets that arrive, are written by
//! hand, since the `rosc` crate is unmaintained.

mod incoming;
mod listen;
mod remote;
#[cfg(test)]
mod tests;

use std::{fmt, net::Ipv4Addr};

pub use listen::{Heard, listen};
pub use remote::Remote;

/// Characters that no host holds, of the names and IPv4 addresses that are
/// taken: those of an IPv6 literal, of a URL, and of a second port, and the
/// null that the resolver rejects
const NOT_IN_HOST: [char; 5] = [':', '[', ']', '/', '\0'];

/// Ends a SLIP frame and starts one too, so that a receiver drops any noise
/// before it
const END: u8 = 0xc0;

const ESC: u8 = 0xdb;

/// Bytes that follow ESC in place of an END and an ESC
const ESCAPED_END: u8 = 0xdc;
const ESCAPED_ESC: u8 = 0xdd;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Destination {
    Tcp(String),
    Udp(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Outgoing {
    pub destination: Destination,
    pub packet: Vec<u8>,
}

/// Writes the destination with its scheme, as `tcp://host:port` or
/// `udp://host:port`, so that the two read alike and name the worker of the
/// destination
impl fmt::Display for Destination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tcp(address) => write!(f, "tcp://{address}"),
            Self::Udp(address) => write!(f, "udp://{address}"),
        }
    }
}

impl Destination {
    /// Reads `host:port`, with or without a `tcp://` before it, or
    /// `udp://host:port`, and returns none for text of any other form, as for
    /// a host that holds a colon, a bracket, or a slash, which no lookup of
    /// an IPv4 address could resolve
    pub fn parse(text: &str) -> Option<Self> {
        let (destination, address) = Self::split(text);
        let (host, port) = address.rsplit_once(':')?;
        (Self::host_is_valid(host) && Self::port_is_valid(port))
            .then(|| destination(address.to_owned()))
    }

    /// Returns whether `text` reads as `host:port` with a good host but a
    /// port that is not a number from 1 to 65535
    pub(super) fn port_is_bad(text: &str) -> bool {
        Self::split(text)
            .1
            .rsplit_once(':')
            .is_some_and(|(host, port)| Self::host_is_valid(host) && !Self::port_is_valid(port))
    }

    fn split(text: &str) -> (fn(String) -> Self, &str) {
        let text = text.trim();
        match text.strip_prefix("udp://") {
            Some(address) => (Self::Udp, address),
            None => (Self::Tcp, text.strip_prefix("tcp://").unwrap_or(text)),
        }
    }

    /// Holds a name or an IPv4 address.  A host of digits and dots must read
    /// as an address in full, since the system resolver also takes a zero
    /// before an octet for octal and fewer than four octets for a short
    /// form, and would send to a machine other than the one that is written.
    fn host_is_valid(host: &str) -> bool {
        !host.is_empty()
            && !host.contains(char::is_whitespace)
            && !host.contains(NOT_IN_HOST)
            && (!reads_as_number(host) || host.parse::<Ipv4Addr>().is_ok())
    }

    pub fn port_is_valid(port: &str) -> bool {
        port.parse::<u16>().is_ok_and(|port| port > 0)
    }
}

pub(super) fn has_open_quote(text: &str) -> bool {
    text.matches('"').count() % 2 == 1
}

/// Returns the OSC message that `text` reads as, or none unless it starts
/// with an address, holds no null, which OSC reads as the end of a string,
/// and closes every double quote that it opens
pub fn packet(text: &str) -> Option<Vec<u8>> {
    if text.contains('\0') || has_open_quote(text) {
        return None;
    }
    let mut words = words(text).into_iter();
    let address = words
        .next()
        .filter(|address| !address.quoted && address.text.starts_with('/'))?;
    let (tags, arguments): (String, Vec<Vec<u8>>) = words.map(argument).unzip();
    Some(
        [padded(&address.text), padded(&format!(",{tags}"))]
            .into_iter()
            .chain(arguments)
            .flatten()
            .collect(),
    )
}

pub(super) fn framed(packet: &[u8]) -> Vec<u8> {
    let escaped = packet.iter().flat_map(|&byte| match byte {
        END => vec![ESC, ESCAPED_END],
        ESC => vec![ESC, ESCAPED_ESC],
        byte => vec![byte],
    });
    [END].into_iter().chain(escaped).chain([END]).collect()
}

fn argument(word: Word) -> (char, Vec<u8>) {
    let Word { text, quoted } = word;
    match (quoted, text.parse::<i32>(), text.parse::<f32>()) {
        (false, Ok(integer), _) => ('i', integer.to_be_bytes().to_vec()),
        // Words such as `inf` and `nan` stay strings
        (false, _, Ok(float)) if float.is_finite() => ('f', float.to_be_bytes().to_vec()),
        _ => ('s', padded(&text)),
    }
}

/// OSC string: the bytes, a null, and more nulls to a multiple of four
fn padded(text: &str) -> Vec<u8> {
    let length = (text.len() / 4 + 1) * 4;
    text.bytes()
        .chain(std::iter::repeat(0))
        .take(length)
        .collect()
}

struct Word {
    text: String,
    quoted: bool,
}

fn words(text: &str) -> Vec<Word> {
    text.split('"')
        .enumerate()
        .flat_map(|(part, between)| {
            // Parts alternate between outside and inside the quotes
            let quoted = part % 2 == 1;
            let texts = if quoted {
                vec![between]
            } else {
                between.split_whitespace().collect()
            };
            texts.into_iter().map(move |text| Word {
                text: text.to_owned(),
                quoted,
            })
        })
        .collect()
}

/// Returns whether the system resolver reads `host` as an address, as it
/// does a host of digits and dots, one whose last label is a number, or one
/// whose last label starts with `0x`, reading a leading zero as octal and
/// `0x` as hexadecimal
fn reads_as_number(host: &str) -> bool {
    let last = host.rsplit('.').next().unwrap_or(host);
    host.chars().all(|c| c.is_ascii_digit() || c == '.')
        || (!last.is_empty() && last.chars().all(|c| c.is_ascii_digit()))
        || last
            .get(..2)
            .is_some_and(|start| start.eq_ignore_ascii_case("0x"))
}
