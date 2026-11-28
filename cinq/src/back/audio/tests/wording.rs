// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the words that tell why a sound file cannot be played

use std::fs;

use tempfile::TempDir;

use super::{silence, silence_in};
use crate::back::audio::probe;

/// Returns why the file that holds `bytes` cannot be played
fn failing(bytes: &[u8]) -> String {
    let directory = TempDir::new().expect("a temporary directory can be made");
    let path = directory.path().join("sound.wav");
    fs::write(&path, bytes).expect("the file can be written");
    probe(&path).expect_err("the file cannot be played")
}

#[test]
fn file_that_no_decoder_reads_is_not_a_sound_file_that_cinq_can_play() {
    let words = "not a sound file that cinq can play";
    assert_eq!(failing(b"not a sound"), words);
    assert_eq!(failing(&[]), words);
    // Header that stops where its format chunk begins
    assert_eq!(failing(&silence(1.0)[..14]), words);
}

#[test]
fn file_that_is_not_there_is_not_found() {
    let directory = TempDir::new().expect("a temporary directory can be made");
    let reason = probe(&directory.path().join("none.wav")).expect_err("the file is not there");
    assert_eq!(reason, "file not found");
}

#[test]
fn wav_file_of_more_than_two_channels_is_refused_as_kira_would_spin_on_it() {
    let words = "holds more than two channels";
    assert_eq!(failing(&silence_in(0.1, 8000, 3)), words);
    assert_eq!(failing(&silence_in(0.1, 8000, 6)), words);
    // Format chunk that comes after a chunk of an odd size and its padding
    let whole = silence_in(0.1, 8000, 6);
    let junk = [b"JUNK".as_slice(), &3_u32.to_le_bytes(), &[0; 4]].concat();
    let size = (whole.len() + junk.len() - 8) as u32;
    let later = [
        &whole[..4],
        &size.to_le_bytes(),
        &whole[8..12],
        &junk,
        &whole[12..],
    ]
    .concat();
    assert_eq!(failing(&later), words);
}

#[test]
fn wav_file_of_one_or_two_channels_is_probed() {
    let directory = TempDir::new().expect("a temporary directory can be made");
    [1, 2].into_iter().for_each(|channels| {
        let path = directory.path().join(format!("{channels}.wav"));
        fs::write(&path, silence_in(0.1, 8000, channels)).expect("the file can be written");
        assert!(probe(&path).is_ok());
    });
}
