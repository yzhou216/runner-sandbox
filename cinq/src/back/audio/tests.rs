// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of making the sounds of audio cues ready ahead, and, in [`heard`],
//! of the time that the sounds of video cues have reached, which need no
//! audio device

mod voice;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
    time::Duration,
};

use tempfile::TempDir;

use super::{
    READY_AHEAD,
    preload::{Ask, Preloaded, Preloader, Wanted, serve},
    upcoming,
};
use crate::back::{
    Advance, Clip, Command, Cue, CueId, Kind, Workspace,
    workspace::tests::{GO, audio, loaded, silent, workspace},
};

fn upcoming_cues(workspace: &Workspace) -> Vec<u32> {
    cues_within(workspace, READY_AHEAD)
}

fn cues_within(workspace: &Workspace, ahead: f64) -> Vec<u32> {
    upcoming(workspace, ahead)
        .into_iter()
        .map(|ask| ask.cue.0)
        .collect()
}

/// Returns the ask for the sound of cue 1 that `wanted` describes, at no
/// change of level
fn ask(wanted: &Wanted) -> Ask {
    Ask {
        cue: CueId(1),
        wanted: wanted.clone(),
        level: 0.0,
    }
}

/// Returns `seconds` of silence as a WAV file: mono, in 16 bits, at 8000
/// frames a second
fn silence(seconds: f32) -> Vec<u8> {
    let data = (seconds * 16000.0) as u32;
    [
        b"RIFF".as_slice(),
        &(36 + data).to_le_bytes(),
        b"WAVE",
        b"fmt ",
        &16_u32.to_le_bytes(),
        &1_u16.to_le_bytes(),
        &1_u16.to_le_bytes(),
        &8000_u32.to_le_bytes(),
        &16000_u32.to_le_bytes(),
        &2_u16.to_le_bytes(),
        &16_u16.to_le_bytes(),
        b"data",
        &data.to_le_bytes(),
        &vec![0; data as usize],
    ]
    .concat()
}

/// Returns the whole of a tenth of a second of sound in `path` as wanted
fn whole(path: &Path) -> Wanted {
    Wanted {
        path: path.to_owned(),
        clip: Clip {
            start: 0.0,
            end: 0.1,
            looping: false,
        },
    }
}

fn silent_file() -> (TempDir, Wanted) {
    let directory = TempDir::new().expect("a temporary directory can be made");
    let file = directory.path().join("silence.wav");
    fs::write(&file, silence(0.1)).expect("the file can be written");
    (directory, whole(&file))
}

/// Returns what `preloader` made for `cue`, once its thread has made it,
/// waiting five seconds at most
fn made(
    preloader: &mut Preloader,
    cue: CueId,
    wanted: &Wanted,
) -> Option<Result<Preloaded, String>> {
    (0..500).find_map(|_| {
        preloader.collect();
        let made = preloader.take(cue, wanted);
        if made.is_none() {
            thread::sleep(Duration::from_millis(10));
        }
        made
    })
}

#[test]
fn sounds_made_ready_are_those_of_the_next_go_and_of_runs_yet_to_sound() {
    let continuing = Cue {
        advance: Advance::Continue,
        post_wait: 2.0.into(),
        ..audio(1)
    };
    let workspace = workspace(vec![continuing, audio(2), audio(3)]).directory("/show");
    assert_eq!(upcoming_cues(&workspace), [1, 2]);
    let fired = workspace.apply(GO);
    assert_eq!(upcoming_cues(&fired), [2, 3]);
    let wanted = upcoming(&fired, READY_AHEAD).remove(0).wanted;
    assert_eq!(wanted.path, PathBuf::from("/show/sound.ogg"));
}

#[test]
fn sound_of_a_video_cue_is_made_ready_unless_its_file_holds_none() {
    let video = |id: u32| {
        Cue::new(CueId(id), id.to_string(), Kind::Video, "Movie")
            .file("movie.mp4")
            .advance(Advance::Continue)
            .build()
    };
    let workspace = workspace(vec![video(1), video(2)]);
    assert_eq!(upcoming_cues(&workspace), [1, 2]);
    let without_sound = workspace.apply(loaded(2, "movie.mp4", silent(10.0)));
    assert_eq!(upcoming_cues(&without_sound), [1]);
}

#[test]
fn sound_wanted_is_made_ready_on_a_thread_of_its_own() {
    let (_directory, wanted) = silent_file();
    let mut preloader = Preloader::new();
    preloader.want(vec![ask(&wanted)]);
    let made = made(&mut preloader, CueId(1), &wanted);
    assert!(matches!(made, Some(Ok(_))));
}

#[test]
fn sound_that_cannot_be_made_ready_tells_why() {
    let wanted = whole(Path::new("/nowhere/gone.wav"));
    let mut preloader = Preloader::new();
    preloader.want(vec![ask(&wanted)]);
    let made = made(&mut preloader, CueId(1), &wanted);
    assert!(matches!(&made, Some(Err(reason)) if reason == "file not found"));
}

#[test]
fn sound_no_longer_wanted_is_let_go() {
    let (_directory, wanted) = silent_file();
    let mut preloader = Preloader::new();
    preloader.want(vec![ask(&wanted)]);
    let other = Ask {
        cue: CueId(2),
        ..ask(&wanted)
    };
    preloader.want(vec![other]);
    // Thread makes what it is asked for in order, so the first sound has
    // come back once the second has
    assert!(made(&mut preloader, CueId(2), &wanted).is_some());
    assert!(preloader.take(CueId(1), &wanted).is_none());
}

#[test]
fn a_run_of_asks_for_one_cue_opens_only_the_last() {
    let (_directory, wanted) = silent_file();
    // A file that is not there makes a result of its own if the thread
    // opens it, which is how an open shows
    let replaced = whole(Path::new("/nowhere/gone.wav"));
    let (asks, waiting) = mpsc::channel();
    [&replaced, &replaced, &wanted]
        .into_iter()
        .for_each(|sound| asks.send(ask(sound)).expect("the receiver is held"));
    drop(asks);
    let (done, made) = mpsc::channel();
    serve(&waiting, &done);
    let made: Vec<_> = made.try_iter().collect();
    assert!(matches!(
        made.as_slice(),
        [(CueId(1), made, Ok(_))] if *made == wanted
    ));
}

#[test]
fn sounds_are_made_ready_only_when_they_start_within_the_window() {
    // Cues 2 and 3 start together, 4 moves on past its pre-wait, so the
    // five start 0, 4, 4, 11, and 16 seconds after GO
    let continuing = |id: u32, pre_wait: f32, post_wait: f32| Cue {
        advance: Advance::Continue,
        pre_wait: pre_wait.into(),
        post_wait: post_wait.into(),
        ..audio(id)
    };
    let chain = workspace(vec![
        continuing(1, 0.0, 4.0),
        continuing(2, 0.0, 0.0),
        continuing(3, 0.0, 6.0),
        continuing(4, 1.0, 5.0),
        audio(5),
    ]);
    let fired = chain.clone().apply(GO);
    let later = fired.clone().apply(Command::Tick(2.0));
    let cases = [
        ("at the instant of GO only", &chain, 0.0, vec![1]),
        ("sounds that start together", &chain, 4.0, vec![1, 2, 3]),
        ("none outside it", &chain, 10.0, vec![1, 2, 3]),
        ("on its edge", &chain, 11.0, vec![1, 2, 3, 4]),
        ("fired, with the first sounding", &fired, 10.0, vec![2, 3]),
        ("fired, as the time passes", &later, 10.0, vec![2, 3, 4]),
    ];
    cases
        .into_iter()
        .for_each(|(name, workspace, ahead, cues)| {
            assert_eq!(cues_within(workspace, ahead), cues, "{name}");
        });
}
