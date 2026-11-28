// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the passes through a clip that loops part of its file, and of
//! the pictures that follow once a devamp cue lets the clip go on

use std::path::Path;

use super::{
    assert_pictures, decoded, from_at,
    movie::{movie, picture_number},
    patiently,
    player::{pictures_after_the_end, shown},
    showing_cues,
};
use crate::back::{
    Clip, Command, Cue, CueId, Kind, Loop, Shown, Video,
    video::{Showing, player::Player},
    workspace::tests::{GO, devamp_cue},
};

/// Returns a clip of the first 0.8 seconds of a video that loops from 0.2 to
/// 0.5 seconds, which a devamp cue lets go on at `released`
fn sectioned(released: Option<f64>) -> Clip {
    Clip {
        start: 0.0,
        end: 0.8,
        looping: Some(Loop {
            start: 0.2,
            end: 0.5,
            released,
        }),
    }
}

/// Asserts that the pictures `seen` end at picture 23 and never go back
fn assert_rising_to_the_last(seen: &[usize]) {
    assert_eq!(seen.last(), Some(&23));
    assert!(
        seen.windows(2).all(|pair| pair[0] < pair[1]),
        "{seen:?} goes back"
    );
}

#[test]
fn first_pass_of_a_sectioned_clip_runs_from_its_start_to_the_end_of_its_loop() {
    let (_directory, path) = movie(24, 4);
    assert_pictures(&decoded(&path, sectioned(None), 0), &from_at(0..=14, 0.0));
}

#[test]
fn loop_pass_runs_round_the_loop_each_due_a_pass_after_the_one_before() {
    let (_directory, path) = movie(24, 4);
    // Pictures 6 to 14 make the loop, which starts after the first pass
    // of 15 pictures, and picture 6 is the first that shows within it, after
    // pictures 4 and 5 that decoding starts with
    assert_pictures(&decoded(&path, sectioned(None), 1), &from_at(6..=14, 15.0));
    assert_pictures(&decoded(&path, sectioned(None), 2), &from_at(6..=14, 24.0));
}

#[test]
fn pass_after_a_release_runs_from_the_end_of_the_loop_to_the_end_of_the_clip() {
    let (_directory, path) = movie(24, 4);
    let released = sectioned(Some(0.8));
    assert_pictures(&decoded(&path, released, 1), &from_at(6..=14, 15.0));
    // Picture 15 shows at the end of the loop, which decoding reaches from
    // picture 12, the last sync sample before it
    assert_pictures(&decoded(&path, released, 2), &from_at(15..=23, 24.0));
    assert!(released.pass(3).is_none());
}

#[test]
fn release_that_lands_on_the_pass_under_way_replaces_the_pass_after_it() {
    let (_directory, path) = movie(24, 4);
    let player = Player::start(path, Shown::Clip(sectioned(None)));
    // The pass under way is the first, which the release makes the one
    // before the last, so that the loop pass decoding ahead is not the one
    // to show after it
    let (player, seen) = shown(player.released(Shown::Clip(sectioned(Some(0.5)))), 0.8, 23);
    assert_rising_to_the_last(&seen);
    // Last pass is the end of the player, which holds no picture after it
    assert_eq!(pictures_after_the_end(player), 0);
}

#[test]
fn player_of_a_clip_that_does_not_loop_ends_after_its_only_pass() {
    let (_directory, path) = movie(15, 5);
    let player = Player::start(path, Shown::Clip(super::clip(0.0, 0.5, false)));
    let (player, seen) = shown(player, 1.0, 14);
    assert_eq!(seen.last(), Some(&14));
    assert_eq!(pictures_after_the_end(player), 0);
}

#[test]
fn player_made_ready_for_a_clip_serves_the_run_whose_clip_a_devamp_cue_has_let_go() {
    let showing = |shown| Showing {
        path: Path::new("/show/movie.mp4").to_owned(),
        shown,
    };
    let ready = showing(Shown::Clip(sectioned(None)));
    assert!(ready.matches(&showing(Shown::Clip(sectioned(Some(0.8))))));
    let shorter = Clip {
        end: 0.7,
        ..sectioned(None)
    };
    assert!(!ready.matches(&showing(Shown::Clip(shorter))));
    assert!(!ready.matches(&showing(Shown::Still)));
    let elsewhere = Showing {
        path: Path::new("/show/other.mp4").to_owned(),
        ..showing(Shown::Clip(sectioned(None)))
    };
    assert!(!ready.matches(&elsewhere));
}

#[test]
fn run_devamped_in_its_pre_wait_shows_the_pass_from_the_end_of_the_loop() {
    let (_directory, path) = movie(24, 4);
    let vamp = Cue::new(CueId(1), "1", Kind::Video, "Movie")
        .file("movie.mp4")
        .looping(true)
        .loop_start(0.2)
        .loop_end(0.6)
        .end(0.8)
        .pre_wait(5.0)
        .build();
    let mut video = Video::default();
    let (waiting, _) = video.sync(showing_cues(&path, vec![vamp, devamp_cue(2, 1)]));
    assert_eq!(video.ready.len(), 1);
    // The run takes the player made ready before the devamp cue acts on it,
    // so that the player has to move on to the clip that has been let go
    let devamped = waiting.apply(GO).apply(GO);
    let (playing, _) = video.sync(devamped);
    assert_eq!((video.ready.len(), video.runs.len()), (0, 1));
    let (_, seen) = patiently(
        (playing.apply(Command::Tick(5.79)), Vec::new()),
        |(workspace, seen): (_, Vec<usize>)| {
            let (workspace, pictures) = video.sync(workspace);
            let numbers = pictures
                .iter()
                .map(|(_, picture)| picture_number(&picture.pixels));
            let seen = [seen, numbers.collect()].concat();
            let done = seen.last() == Some(&23);
            ((workspace, seen), done)
        },
    );
    assert_rising_to_the_last(&seen);
}
