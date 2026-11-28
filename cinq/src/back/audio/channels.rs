// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Channels of a WAV file, which its header tells
//!
//! kira decodes mono and stereo sound alone.  Given more channels, its
//! decoder fails, reaches the end of the file, and then asks for frames of
//! it again without end, so that its thread spins a core for good, even once
//! the sound is let go.  The sound has to be refused before kira sees it, and
//! kira does not tell the channels of a file, so the header is read here.

use std::{
    fs::File,
    io::{self, Read},
    iter,
    path::Path,
};

/// Bytes of the start of a file that hold the format chunk of a WAV file, as
/// they do unless chunks of other kinds of an unusual size come before it
const HEAD: u64 = 64 * 1024;

/// Returns the channels that the header of the WAV file at `path` names, or
/// none for a file that is not WAV or whose header holds no format chunk
pub(super) fn of_wav(path: &Path) -> io::Result<Option<u16>> {
    let mut head = Vec::new();
    File::open(path)?.take(HEAD).read_to_end(&mut head)?;
    Ok(wav_channels(&head))
}

/// Returns the channels that the format chunk of a WAV file at the start of
/// `head` names
fn wav_channels(head: &[u8]) -> Option<u16> {
    let chunks = head
        .strip_prefix(b"RIFF")?
        .get(4..)?
        .strip_prefix(b"WAVE")?;
    iter::successors(Some(chunks), |chunk| {
        let size = u32::from_le_bytes(chunk.get(4..8)?.try_into().ok()?) as usize;
        // Chunks hold an even number of bytes, and a size says how many
        // before the byte that pads an odd one
        chunk.get(size.saturating_add(8 + size % 2)..)
    })
    .find(|chunk| chunk.starts_with(b"fmt "))
    .and_then(|chunk| Some(u16::from_le_bytes(chunk.get(10..12)?.try_into().ok()?)))
}
