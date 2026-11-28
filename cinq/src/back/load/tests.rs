// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of loading the files of audio and video cues, which read each file
//! as the kind of its cue plays it

mod recheck;
mod watch;

use std::{fs, path::Path};

use super::{Load, load_files, video_sound_reason};
use crate::back::{
    Command, Cue, CueId, Kind, Media,
    audio::{
        NOT_SOUND,
        tests::{silence, silence_at},
    },
    silent,
    video::tests::movie::movie,
    workspace::tests::fresh,
};

fn playing(id: u32, kind: Kind, file: &str) -> Cue {
    Cue::new(CueId(id), id.to_string(), kind, "Cue")
        .file(file)
        .build()
}

/// Returns what the loaded commands found, with the number of the cue that
/// each finding belongs to
fn findings(commands: Vec<Command>) -> Vec<(u32, Media)> {
    commands
        .into_iter()
        .filter_map(|command| match command {
            Command::Loaded { cue, media, .. } => Some((cue.0, media)),
            _ => None,
        })
        .collect()
}

fn found(cues: Vec<Cue>, directory: &Path) -> Vec<(u32, Media)> {
    let workspace = fresh(cues).directory(directory);
    findings(load_files(Load::all(&workspace, workspace.unloaded())))
}

#[test]
fn every_cue_that_names_a_missing_file_learns_that_it_is_not_found() {
    let cues = vec![
        playing(1, Kind::Audio, "sound.wav"),
        playing(2, Kind::Audio, "sound.wav"),
        playing(3, Kind::Video, "movie.mp4"),
    ];
    let not_found = Media::Failed("file not found".to_owned());
    let expected: Vec<(u32, Media)> = (1..=3).map(|cue| (cue, not_found.clone())).collect();
    assert_eq!(found(cues, Path::new("/nowhere")), expected);
}

#[test]
fn video_file_loads_as_a_picture_for_a_video_cue_and_as_no_sound_for_an_audio_cue() {
    let (directory, _) = movie(15, 5);
    let cues = vec![
        playing(1, Kind::Video, "movie.mp4"),
        playing(2, Kind::Audio, "movie.mp4"),
    ];
    let found = found(cues, directory.path());
    assert_eq!(found[0], (1, silent(0.5)));
    assert!(matches!(found[1], (2, Media::Failed(_))), "{found:?}");
}

#[cfg(unix)]
#[test]
fn file_that_panics_the_reader_fails_instead_of_ending_the_load() {
    // A buffered reader panics where a file reports position zero after each read
    let found = found(vec![playing(1, Kind::Video, "/dev/zero")], Path::new("/"));
    assert!(matches!(found[..], [(1, Media::Failed(_))]), "{found:?}");
}

#[test]
fn sound_file_that_panics_the_decoder_fails_alone_and_the_others_load() {
    let directory = tempfile::tempdir().expect("a directory can be made");
    // Header of a sound that plays at no frames a second
    fs::write(directory.path().join("rateless.wav"), silence_at(0.1, 0))
        .expect("the file can be written");
    fs::write(directory.path().join("good.wav"), silence(0.1)).expect("the file can be written");
    let cues = vec![
        playing(1, Kind::Audio, "rateless.wav"),
        playing(2, Kind::Audio, "good.wav"),
    ];
    let found = found(cues, directory.path());
    assert_eq!(found[0], (1, Media::Failed("cannot be read".to_owned())));
    assert!(matches!(found[1], (2, Media::Loaded { .. })), "{found:?}");
}

#[test]
fn sound_track_of_a_video_that_no_decoder_reads_is_blamed_on_its_format() {
    assert_eq!(
        video_sound_reason(NOT_SOUND),
        "its sound is in a format that cinq cannot play"
    );
    assert_eq!(
        video_sound_reason("file not found"),
        "its sound cannot play: file not found"
    );
}
