// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Helpers that take the boxes of a test video apart and put them together
//! again, for the tests of files that [`movie`](super::movie) does not
//! write

use mp4_atom::{Decode, Encode, Mdia, Minf, Moov, Stbl, Trak};

/// Box of an MP4 file, as its type and the bytes that follow its header
pub(super) type Part = ([u8; 4], Vec<u8>);

/// Returns the boxes that `bytes` holds one after another, whose headers
/// hold their sizes in four bytes
pub(super) fn split(bytes: &[u8]) -> Vec<Part> {
    let size = bytes
        .first_chunk::<4>()
        .map(|size| u32::from_be_bytes(*size) as usize)
        .filter(|&size| size >= 8 && size <= bytes.len());
    size.map_or_else(Vec::new, |size| {
        let kind = bytes[4..8].try_into().expect("the type is four bytes");
        let rest = split(&bytes[size..]);
        [vec![(kind, bytes[8..size].to_vec())], rest].concat()
    })
}

/// Returns `part` with a header of eight bytes
pub(super) fn narrow((kind, body): &Part) -> Vec<u8> {
    let size = u32::try_from(body.len() + 8).expect("the box is small");
    [&size.to_be_bytes()[..], kind, body].concat()
}

/// Returns `bytes`, boxes one after another, with the body of the box that
/// `path` leads to, through the boxes of each type in turn, replaced by
/// `with` of the body, and the sizes of the boxes that hold it made right
pub(super) fn reworked(
    bytes: &[u8],
    path: &[&[u8; 4]],
    with: &impl Fn(Vec<u8>) -> Vec<u8>,
) -> Vec<u8> {
    let Some((next, rest)) = path.split_first() else {
        return bytes.to_vec();
    };
    split(bytes)
        .into_iter()
        .flat_map(|(kind, body)| {
            let body = match (&kind == *next, rest.is_empty()) {
                (false, _) => body,
                (true, true) => with(body),
                (true, false) => reworked(&body, rest, with),
            };
            narrow(&(kind, body))
        })
        .collect()
}

/// Returns `part` with a header of sixteen bytes, which holds its size in
/// eight
pub(super) fn wide((kind, body): &Part) -> Vec<u8> {
    let size = (body.len() + 16) as u64;
    [&1_u32.to_be_bytes()[..], kind, &size.to_be_bytes(), body].concat()
}

/// Returns `moov`, the bytes of a moov box after its header, with each
/// chunk of samples `by` bytes further into the file
pub(super) fn shifted(moov: &[u8], by: usize) -> Vec<u8> {
    let at = moov
        .windows(4)
        .position(|window| window == b"stco")
        .expect("the box is there")
        + 8;
    let count = u32::from_be_bytes(moov[at..at + 4].try_into().expect("four bytes")) as usize;
    let offsets = moov[at + 4..]
        .chunks(4)
        .take(count)
        .flat_map(|offset| {
            let offset = u32::from_be_bytes(offset.try_into().expect("four bytes"));
            (offset + by as u32).to_be_bytes()
        })
        .collect::<Vec<_>>();
    [&moov[..at + 4], &offsets, &moov[at + 4 + offsets.len()..]].concat()
}

/// Returns the body of a `udta` box that holds a `meta` box as ffmpeg
/// writes it, with a handler of `mdir` and an empty `ilst`
pub(super) fn ilst_udta() -> Vec<u8> {
    let handler = [&[0; 8][..], b"mdir", &[0; 13]].concat();
    let meta = [
        vec![0; 4],
        narrow(&(*b"hdlr", handler)),
        narrow(&(*b"ilst", vec![])),
    ]
    .concat();
    narrow(&(*b"meta", meta))
}

/// Returns `bytes`, the boxes of a test video, with the tables of the track
/// in its moov box replaced by `revise` of them
pub(super) fn revised(bytes: &[u8], revise: &impl Fn(Stbl) -> Stbl) -> Vec<u8> {
    let track = |trak: Trak| Trak {
        mdia: Mdia {
            minf: Minf {
                stbl: revise(trak.mdia.minf.stbl),
                ..trak.mdia.minf
            },
            ..trak.mdia
        },
        ..trak
    };
    reworked(bytes, &[b"moov"], &|body| {
        let bytes = narrow(&(*b"moov", body));
        let moov = Moov::decode(&mut bytes.as_slice()).expect("the movie decodes");
        let mut encoded = Vec::new();
        Moov {
            trak: moov.trak.into_iter().map(track).collect(),
            ..moov
        }
        .encode(&mut encoded)
        .expect("the movie encodes");
        encoded[8..].to_vec()
    })
}
