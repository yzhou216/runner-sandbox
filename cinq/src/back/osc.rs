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

#[cfg(test)]
mod tests;

use std::fmt;

/// Byte that ends a SLIP frame, and starts one too, so that a receiver
/// drops whatever noise came before it
const END: u8 = 0xc0;

const ESC: u8 = 0xdb;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Destination {
    Tcp(String),
    Udp(String),
}

/// Packet of a message on its way to its destination
#[derive(Debug, Clone, PartialEq)]
pub struct Outgoing {
    pub destination: Destination,
    pub packet: Vec<u8>,
}

impl fmt::Display for Destination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tcp(address) => f.write_str(address),
            Self::Udp(address) => write!(f, "udp://{address}"),
        }
    }
}

impl Destination {
    /// Returns the destination that `text` names, by a host, which can be
    /// a name or an IPv4 address, and a port
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let (udp, address) = match text.strip_prefix("udp://") {
            Some(address) => (true, address),
            None => (false, text.strip_prefix("tcp://").unwrap_or(text)),
        };
        let (host, port) = address.rsplit_once(':')?;
        let valid = !host.is_empty()
            && !host.contains(char::is_whitespace)
            && port.parse::<u16>().is_ok_and(|port| port > 0);
        let address = address.to_owned();
        valid.then_some(if udp {
            Self::Udp(address)
        } else {
            Self::Tcp(address)
        })
    }
}

/// Returns the packet of the message that `text` writes, or none when it
/// does not start with an address
pub fn packet(text: &str) -> Option<Vec<u8>> {
    let mut words = words(text).into_iter();
    let address = words
        .next()
        .filter(|address| !address.quoted && address.text.starts_with('/'))?;
    let (tags, arguments): (String, Vec<Vec<u8>>) = words.map(argument).unzip();
    Some(
        [padded(&address.text), padded(&format!(",{tags}"))]
            .into_iter()
            .chain(arguments)
            .collect::<Vec<_>>()
            .concat(),
    )
}

/// Returns `packet` as a SLIP frame, between END bytes, with the END and
/// ESC bytes within it escaped
pub fn framed(packet: &[u8]) -> Vec<u8> {
    let escaped = packet.iter().flat_map(|&byte| match byte {
        END => vec![ESC, 0xdc],
        ESC => vec![ESC, 0xdd],
        byte => vec![byte],
    });
    [END].into_iter().chain(escaped).chain([END]).collect()
}

/// Returns the type tag and the bytes of an argument, which is a string
/// whenever quotes hold it
fn argument(word: Word) -> (char, Vec<u8>) {
    let Word { text, quoted } = word;
    match (text.parse::<i32>(), text.parse::<f32>()) {
        _ if quoted => ('s', padded(&text)),
        (Ok(integer), _) => ('i', integer.to_be_bytes().to_vec()),
        // Words such as `inf` and `nan` stay strings
        (_, Ok(float)) if float.is_finite() => ('f', float.to_be_bytes().to_vec()),
        _ => ('s', padded(&text)),
    }
}

/// Returns `text` as an OSC string: its bytes, ended by a null and padded
/// with more to a multiple of four
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

/// Returns the words of `text`: what double quotes hold, whole, and the
/// rest split at spaces
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
