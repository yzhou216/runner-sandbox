// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The tracks of an MP4 file, read from its moov box without decoding what
//! cinq does not use
//!
//! mp4-atom refuses a box that holds a box it cannot read, so that decoding
//! the moov box whole refuses a file whose tags are damaged, though video
//! cues play without them.  The boxes inside the moov box are walked by
//! their headers instead, and only the tracks are decoded, except those of
//! sound, whose handler is enough to know that the file holds sound.

use std::{
    collections::BTreeSet,
    io::{Cursor, Read, Seek, SeekFrom},
    iter,
    ops::Range,
};

use mp4_atom::{
    Atom, Avcc, Codec, DecodeAtom, Error, FourCC, Hdlr, Header, Mdia, Moov, ReadFrom, Trak,
};

/// Handler of the tracks that hold sound
const SOUND: FourCC = FourCC::new(b"soun");

/// What cinq reads of the moov box of a file
#[derive(Default)]
pub(super) struct Movie {
    /// H.264 track with the lowest identity, with its configuration
    pub(super) video: Option<(Trak, Avcc)>,
    /// Whether the file holds a sound track
    pub(super) sound: bool,
    /// Whether a track that cannot be read may have held the picture
    pub(super) lost: bool,
    /// What was skipped because it cannot be read, as the words that say so
    pub(super) skipped: BTreeSet<String>,
}

/// Reads the moov box of the MP4 file in `file`, which is `length` bytes
/// long, or returns none when the file holds none that fits in it.  The
/// boxes before it are skipped by their sizes.
pub(super) fn read<R: Read + Seek>(file: &mut R, length: u64) -> Option<Movie> {
    let (_, body) = boxes(file, 0, length).find(|(kind, _)| *kind == Moov::KIND)?;
    file.seek(SeekFrom::Start(body.start)).ok()?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(body.end - body.start)
        .read_to_end(&mut bytes)
        .ok()?;
    let movie = boxes(&mut Cursor::new(&bytes[..]), 0, bytes.len() as u64)
        .filter(|(kind, _)| *kind == Trak::KIND)
        .map(|(_, body)| within(&bytes, body))
        .fold(Movie::default(), Movie::with);
    Some(movie)
}

/// Returns the configuration of H.264 that the description of `trak` holds,
/// which makes the track one of H.264
pub(super) fn avcc(trak: &Trak) -> Option<&Avcc> {
    trak.mdia
        .minf
        .stbl
        .stsd
        .codecs
        .iter()
        .find_map(|codec| match codec {
            Codec::Avc1(avc) => Some(&avc.avcc),
            _ => None,
        })
}

impl Movie {
    /// Returns the movie once the track whose bytes after its header are
    /// `trak` is in it
    fn with(self, trak: &[u8]) -> Self {
        match handler(trak) {
            Some(SOUND) => Self {
                sound: true,
                ..self
            },
            _ => match decoded::<Trak>(trak) {
                Ok(trak) => Self {
                    video: self
                        .video
                        .into_iter()
                        .chain(avcc(&trak).cloned().map(|avc| (trak, avc)))
                        .min_by_key(|(trak, _)| trak.tkhd.track_id),
                    ..self
                },
                Err(error) => Self {
                    lost: true,
                    skipped: self
                        .skipped
                        .into_iter()
                        .chain([format!("skipped a track, which cannot be read: {error}")])
                        .collect(),
                    ..self
                },
            },
        }
    }
}

/// Decodes the box of type `T` whose bytes after its header are `bytes`
fn decoded<T: Atom>(bytes: &[u8]) -> Result<T, Error> {
    let header = Header {
        kind: T::KIND,
        size: Some(bytes.len()),
    };
    T::decode_atom(&header, &mut &bytes[..])
}

/// Returns the handler of the track whose bytes after its header are
/// `trak`, which names its kind of media, found without decoding the rest
/// of the track
fn handler(trak: &[u8]) -> Option<FourCC> {
    let hdlr = inside(inside(trak, Mdia::KIND)?, Hdlr::KIND)?;
    // Version and flags and a reserved word come before the handler
    let handler = hdlr.get(8..12)?.try_into().ok()?;
    Some(FourCC::new(handler))
}

/// Returns the bytes after the header of the first box of `kind` in `bytes`
fn inside(bytes: &[u8], kind: FourCC) -> Option<&[u8]> {
    boxes(&mut Cursor::new(bytes), 0, bytes.len() as u64)
        .find(|(found, _)| *found == kind)
        .map(|(_, body)| within(bytes, body))
}

/// Returns the bytes of `body`, which a walk of `bytes` found
fn within(bytes: &[u8], body: Range<u64>) -> &[u8] {
    &bytes[body.start as usize..body.end as usize]
}

/// Returns the type and the body, after its header, of each box that lies
/// one after another between the offsets `from` and `end` of `file`.  A box
/// that claims no bytes runs to `end`.  The walk stops at a box that runs
/// past `end`, so that no body lies outside, and every box starts after the
/// one before it, so that a file whose boxes lead in circles ends too.
fn boxes<R: Read + Seek>(
    file: &mut R,
    from: u64,
    end: u64,
) -> impl Iterator<Item = (FourCC, Range<u64>)> + '_ {
    let mut next = from;
    iter::from_fn(move || {
        file.seek(SeekFrom::Start(next)).ok()?;
        let header = Option::<Header>::read_from(file).ok()??;
        let start = file.stream_position().ok()?;
        let last = header
            .size
            .map_or(Some(end), |size| start.checked_add(size as u64))
            .filter(|last| (start..=end).contains(last))?;
        next = last;
        Some((header.kind, start..last))
    })
}
