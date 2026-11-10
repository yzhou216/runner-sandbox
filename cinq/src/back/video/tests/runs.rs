// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the video output following the runs of video cues: the
//! pictures it hands on, those it makes ready ahead, and the files whose
//! picture cannot play

use std::{
    fs,
    ops::ControlFlow,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use super::{
    GO, PATIENCE,
    movie::{broken_movie, movie, picture_number},
    repeated, showing_cue, synced,
};
use crate::back::{
    Command, Cue, CueId, Kind, Media, Video, Workspace, video::decoder::UNDECODABLE,
};

fn showing(path: &Path) -> Workspace {
    showing_cue(
        path,
        Cue::new(CueId(1), "1", Kind::Video, "Movie")
            .file("movie.mp4")
            .build(),
    )
}

#[test]
fn video_output_shows_the_pictures_of_a_run_and_lets_its_thread_go_once_it_stops() {
    let (_directory, path) = movie(15, 5);
    let mut video = Video::default();
    let playing = showing(&path).apply(GO);
    let (playing, picture) = synced(&mut video, playing);
    let (cue, picture) = picture.expect("the first picture comes due");
    assert_eq!((cue, picture_number(&picture.pixels)), (CueId(1), 0));
    assert_eq!(video.runs.len(), 1);
    let (_, pictures) = video.sync(playing.apply(Command::Stop(CueId(1))));
    assert!(pictures.is_empty());
    assert!(video.runs.is_empty());
}

/// Syncs `video` with `workspace`, which keeps its clock where it is, until
/// the next picture has arrived, and returns when it is due
fn due(video: &mut Video, workspace: Workspace) -> (Workspace, Option<f64>) {
    let started = Instant::now();
    repeated(workspace, |workspace| {
        let (workspace, _) = video.sync(workspace);
        match video.next_due(&workspace) {
            None if started.elapsed() < PATIENCE => {
                thread::sleep(Duration::from_millis(5));
                ControlFlow::Continue(workspace)
            }
            next => ControlFlow::Break((workspace, next)),
        }
    })
}

#[test]
fn next_picture_of_a_run_comes_due_a_picture_after_the_one_it_shows() {
    let (_directory, path) = movie(15, 5);
    let mut video = Video::default();
    let (playing, _) = synced(&mut video, showing(&path).apply(GO));
    let (playing, next) = due(&mut video, playing);
    assert!(
        next.is_some_and(|next| (next * 30.0 - 1.0).abs() < 1e-6),
        "{next:?}"
    );
    let (paused, _) = video.sync(playing.apply(Command::Pause(true)));
    assert_eq!(video.next_due(&paused), None);
}

#[test]
fn picture_that_the_next_go_starts_is_made_ready_ahead_for_its_run_to_take() {
    let (_directory, path) = movie(15, 5);
    let mut video = Video::default();
    let (waiting, _) = video.sync(showing(&path));
    assert_eq!((video.ready.len(), video.runs.len()), (1, 0));
    let (playing, early) = video.sync(waiting.apply(GO));
    assert_eq!((video.ready.len(), video.runs.len()), (0, 1));
    // The picture may have decoded since it was made ready, so that this
    // sync hands it on, and no later one does while the clock stands still
    let picture = early
        .into_iter()
        .next()
        .or_else(|| synced(&mut video, playing).1);
    assert!(picture.is_some());
    // List played to its end leaves nothing to make ready
    assert!(video.ready.is_empty());
}

#[test]
fn run_that_loops_a_picture_that_cannot_be_decoded_ends_with_a_warning() {
    let (_directory, path) = broken_movie(6, 3, &[0, 1, 2, 3, 4, 5]);
    let looping = Cue::new(CueId(1), "1", Kind::Video, "Movie")
        .file("movie.mp4")
        .looping(true)
        .build();
    let mut video = Video::default();
    let playing = showing_cue(&path, looping).apply(GO);
    let (ended, picture) = synced(&mut video, playing);
    assert!(picture.is_none());
    let cue = ended.cue(CueId(1)).expect("the cue is there");
    assert_eq!(cue.media, Media::Failed(UNDECODABLE.to_owned()));
    assert!(ended.active().is_empty());
}

#[test]
fn run_whose_video_file_has_gone_since_it_loaded_ends_as_a_file_that_cannot_play() {
    let (_directory, path) = movie(15, 5);
    let loaded = showing(&path);
    fs::remove_file(&path).expect("the video can be removed");
    let mut video = Video::default();
    let (ended, _) = synced(&mut video, loaded.apply(GO));
    let cue = ended.cue(CueId(1)).expect("the cue is there");
    assert_eq!(cue.media, Media::Failed("file not found".to_owned()));
    assert!(ended.active().is_empty());
}

#[test]
fn video_file_without_sound_plays_its_picture_alone() {
    let (_directory, path) = movie(15, 5);
    let playing = showing(&path).apply(GO);
    let active = playing.active_of(CueId(1)).expect("the cue runs");
    assert!(active.shows_picture());
    assert_eq!(active.sound(), None);
    assert_eq!(playing.problems().count(), 0);
}

#[test]
fn video_cue_whose_file_cannot_load_cannot_play() {
    let workspace = showing(Path::new("/nowhere/movie.mp4"));
    let cue = workspace.cue(CueId(1)).expect("the cue is there");
    assert_eq!(cue.media, Media::Failed("file not found".to_owned()));
    assert!(workspace.apply(GO).active().is_empty());
}
