// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the thread that makes the sounds wanted ready

use std::{fs, path::Path, sync::mpsc};

use tempfile::TempDir;

use super::{ask, collecting, made, silence, silent_file, whole};
use crate::back::{
    CueId,
    audio::sound::preload::{Ask, Preloader, serve},
};

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
fn sound_that_fails_to_decode_while_it_waits_is_let_go() {
    let directory = TempDir::new().expect("a temporary directory can be made");
    let file = directory.path().join("cut.wav");
    // A file whose header promises more than it holds, as a copy cut short does
    let mut bytes = silence(0.1);
    bytes.truncate(44 + 400);
    fs::write(&file, bytes).expect("the file can be written");
    let wanted = whole(&file);
    let mut preloader = Preloader::new();
    preloader.want(vec![ask(&wanted)]);
    let failed = collecting(&mut preloader, |preloader| preloader.failed(CueId(1)));
    assert!(failed.is_some(), "the sound is let go once it fails");
}
