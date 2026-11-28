// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading the OSC packets that arrive
//!
//! A packet is a message or a bundle.  A message is an address, a type tag
//! string that begins with a comma, and the arguments that it tells, each
//! padded to a multiple of four bytes.  A bundle is `#bundle`, a time tag,
//! and elements, each a size and a packet.  A bundle reads as the messages
//! inside it, in order, whatever its time tag says, since a command that
//! is sent is a command to act on at once.

#[cfg(test)]
mod tests;

use std::iter;

/// Deepest nesting of bundles that is read, so that a packet of bundles
/// inside bundles cannot exhaust the stack
const NESTING: usize = 8;

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub address: String,
    pub arguments: Vec<Argument>,
}

/// Argument of a message, of which only the numbers and booleans that can
/// say off matter, of either width
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Argument {
    Int(i64),
    Float(f64),
    Bool(bool),
    /// Any other argument, which is skipped over
    Other,
}

impl Argument {
    /// Returns whether the argument is a zero or false, as a button of a
    /// remote sends when it is released
    pub(super) fn is_off(self) -> bool {
        matches!(self, Self::Int(0) | Self::Bool(false)) || self == Self::Float(0.0)
    }
}

/// Returns the messages that `bytes` holds, or none if it is not a valid
/// OSC packet
pub fn packet(bytes: &[u8]) -> Option<Vec<Message>> {
    contents(bytes, NESTING)
}

fn contents(bytes: &[u8], nesting: usize) -> Option<Vec<Message>> {
    match bytes.strip_prefix(b"#bundle\0") {
        // Eight bytes of time tag
        Some(bundle) => elements(bundle.get(8..)?)
            .map(|element| contents(element?, nesting.checked_sub(1)?))
            .collect::<Option<Vec<_>>>()
            .map(|messages| messages.concat()),
        None => message(bytes).map(|message| vec![message]),
    }
}

/// Returns the elements of a bundle, each none where the bundle ends short
fn elements(mut rest: &[u8]) -> impl Iterator<Item = Option<&[u8]>> {
    iter::from_fn(move || {
        if rest.is_empty() {
            return None;
        }
        let element = sized(rest);
        rest = element.map_or(&[], |(_, after)| after);
        Some(element.map(|(element, _)| element))
    })
}

/// Returns the element that `bytes` begins with, and what follows it
fn sized(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    let (size, rest) = bytes.split_first_chunk::<4>()?;
    let size = usize::try_from(i32::from_be_bytes(*size)).ok()?;
    Some((rest.get(..size)?, &rest[size..]))
}

fn message(bytes: &[u8]) -> Option<Message> {
    let (address, rest) = string(bytes)?;
    if !address.starts_with('/') {
        return None;
    }
    // A message without a type tag string, as OSC 1.0 allowed, has no
    // arguments that cinq reads
    let arguments = match string(rest) {
        Some((tags, data)) if tags.starts_with(',') => arguments(&tags[1..], data)?,
        _ => Vec::new(),
    };
    Some(Message {
        address: address.to_owned(),
        arguments,
    })
}

fn arguments(tags: &str, data: &[u8]) -> Option<Vec<Argument>> {
    tags.chars()
        .try_fold((Vec::new(), data), |(mut arguments, data), tag| {
            let (argument, rest) = argument(tag, data)?;
            arguments.push(argument);
            Some((arguments, rest))
        })
        .map(|(arguments, _)| arguments)
}

/// Returns the argument that `tag` tells at the start of `data`, and what
/// follows it, or none for a tag that is not known, whose size cannot be
/// told
fn argument(tag: char, data: &[u8]) -> Option<(Argument, &[u8])> {
    match tag {
        'i' => {
            let (number, rest) = data.split_first_chunk::<4>()?;
            Some((Argument::Int(i32::from_be_bytes(*number).into()), rest))
        }
        'f' => {
            let (number, rest) = data.split_first_chunk::<4>()?;
            Some((Argument::Float(f32::from_be_bytes(*number).into()), rest))
        }
        'h' => {
            let (number, rest) = data.split_first_chunk::<8>()?;
            Some((Argument::Int(i64::from_be_bytes(*number)), rest))
        }
        'd' => {
            let (number, rest) = data.split_first_chunk::<8>()?;
            Some((Argument::Float(f64::from_be_bytes(*number)), rest))
        }
        // Character, color, and MIDI
        'c' | 'r' | 'm' => skipped(data, 4),
        // Time tag
        't' => skipped(data, 8),
        's' | 'S' => string(data).map(|(_, rest)| (Argument::Other, rest)),
        'b' => {
            let (size, rest) = sized(data)?;
            let padding = (4 - size.len() % 4) % 4;
            Some((Argument::Other, rest.get(padding..)?))
        }
        'T' => Some((Argument::Bool(true), data)),
        'F' => Some((Argument::Bool(false), data)),
        // Nil, impulse, and the brackets of an array
        'N' | 'I' | '[' | ']' => Some((Argument::Other, data)),
        _ => None,
    }
}

fn skipped(data: &[u8], size: usize) -> Option<(Argument, &[u8])> {
    Some((Argument::Other, data.get(size..)?))
}

/// Returns the OSC string at the start of `bytes`, and what follows its
/// padding
fn string(bytes: &[u8]) -> Option<(&str, &[u8])> {
    let end = bytes.iter().position(|&byte| byte == 0)?;
    let text = str::from_utf8(&bytes[..end]).ok()?;
    Some((text, bytes.get((end / 4 + 1) * 4..)?))
}
