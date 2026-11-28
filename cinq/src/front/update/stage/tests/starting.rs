// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the stage window as GO fires a picture or video cue through a
//! start cue

use std::net::{IpAddr, Ipv4Addr};

use iced::window;

use super::closing;
use crate::{
    back::{Advance, Cue, CueId, Kind, Remote, fresh, loaded, memo, silent, video},
    front::{App, Message},
};

/// Returns the state of the interface with `cues`, the video cue among them
/// loaded, and standby on the first
fn show(cues: Vec<Cue>) -> App {
    let workspace = fresh(cues).apply(loaded(3, "movie.mp4", silent(10.0)));
    App {
        workspace,
        main: Some(window::Id::unique()),
        ..App::default()
    }
}

fn start(id: u32, target: u32) -> Cue {
    Cue::new(CueId(id), id.to_string(), Kind::Start, "Start")
        .target(CueId(target))
        .build()
}

#[test]
fn closed_stage_window_opens_again_as_a_start_cue_fires_a_video_cue() {
    let (open, _) = show(vec![start(1, 3), video(3)]).staged();
    let (gone, _) = closing(open).handled(Message::Go);
    let (reopened, opening) = gone.staged();
    assert!(reopened.stage.window().is_some());
    assert_eq!(opening.units(), 1);
}

#[test]
fn start_cues_that_start_each_other_still_find_the_video_cue_they_fire() {
    let second = Cue {
        advance: Advance::Continue,
        ..start(2, 1)
    };
    let (open, _) = show(vec![start(1, 2), second, video(3)]).staged();
    let (gone, _) = open.handled(Message::Go);
    let (_, raising) = gone.staged();
    assert_eq!(raising.units(), 2);
}

#[test]
fn stage_window_stays_as_it_is_when_a_start_cue_fires_no_picture_or_video_cue() {
    let (open, _) = show(vec![start(1, 2), memo(2), video(3)]).staged();
    let (gone, _) = open.handled(Message::Go);
    let (_, raising) = gone.staged();
    assert_eq!(raising.units(), 0);
}

#[test]
fn stage_window_stays_as_it_is_when_a_start_cue_only_resumes_a_video_cue() {
    let (open, _) = show(vec![video(3), start(1, 3)]).staged();
    let (gone, _) = open.handled(Message::Go);
    let (shown, _) = gone.staged();
    let (paused, _) = shown.handled(Message::PauseAll);
    let (resumed, _) = paused.handled(Message::Go);
    let (_, raising) = resumed.staged();
    assert_eq!(raising.units(), 0);
}

/// Returns the state with a video cue that plays and is paused, and the
/// stage window open, which the operator may have closed
fn paused(closed: bool) -> App {
    let (open, _) = show(vec![video(3)]).staged();
    let (gone, _) = open.handled(Message::Go);
    let (shown, _) = gone.staged();
    let (paused, _) = shown.handled(Message::PauseAll);
    if closed { closing(paused) } else { paused }
}

/// Returns the state once OSC starts the video cue
fn osc_started(app: App) -> App {
    let from = IpAddr::V4(Ipv4Addr::LOCALHOST);
    app.handled(Message::Remote(from, Remote::Start("3".to_owned())))
        .0
}

#[test]
fn stage_window_stays_as_it_is_when_osc_only_resumes_a_paused_video_cue() {
    [false, true].into_iter().for_each(|closed| {
        let resumed = osc_started(paused(closed));
        assert!(!resumed.workspace.active()[0].paused);
        let (resumed, raising) = resumed.staged();
        let window = resumed.stage.window().is_some();
        assert_eq!((window, raising.units()), (!closed, 0), "closed: {closed}");
    });
}
