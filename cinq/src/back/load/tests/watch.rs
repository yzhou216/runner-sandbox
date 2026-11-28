// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the watch over the files that cues have read, which wakes the
//! interface only for a file that is not as its cue saw it

use std::{fs, thread, time::Duration};

use iced::futures::{FutureExt, StreamExt};

use super::recheck::{directory, looked_at, read};
use crate::back::{
    Workspace,
    audio::tests::silence,
    load::{Watch, watch},
};

fn watched(workspace: &Workspace) -> Vec<Watch> {
    Watch::all(workspace, workspace.to_recheck())
}

/// Returns whether the watch of the files that have been read wakes within
/// a few intervals
fn wakes(workspace: &Workspace) -> bool {
    let every = Duration::from_millis(10);
    let mut woken = watch(watched(workspace), every).boxed();
    thread::sleep(every * 20);
    woken.next().now_or_never().flatten().is_some()
}

#[test]
fn files_that_stay_as_they_were_never_wake_the_watch() {
    let directory = directory();
    assert!(!wakes(&read(&directory, &["good.wav", "fake.wav"])));
}

#[test]
fn file_that_changes_wakes_the_watch() {
    let directory = directory();
    let workspace = read(&directory, &["good.wav"]);
    fs::write(directory.path().join("good.wav"), silence(2.0)).expect("the file can be written");
    assert!(wakes(&workspace));
}

#[test]
fn file_that_goes_wakes_the_watch_once_and_wakes_it_again_when_it_comes_back() {
    let directory = directory();
    let workspace = read(&directory, &["good.wav"]);
    let path = directory.path().join("good.wav");
    let kept = fs::read(&path).expect("the file can be read");
    fs::remove_file(&path).expect("the file can be removed");
    assert!(wakes(&workspace));
    let gone = looked_at(workspace);
    assert!(!wakes(&gone));
    fs::write(&path, kept).expect("the file can be written");
    assert!(wakes(&gone));
}

#[test]
fn reading_a_file_anew_gives_the_next_watch_another_identity() {
    let directory = directory();
    let workspace = read(&directory, &["good.wav"]);
    let before = watched(&workspace);
    fs::remove_file(directory.path().join("good.wav")).expect("the file can be removed");
    assert_ne!(watched(&looked_at(workspace)), before);
}
