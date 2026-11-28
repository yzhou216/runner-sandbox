// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the stage window: when it opens, what closing it and closing
//! the main window do, whether iced draws it once minimized, the pictures
//! it keeps, going fullscreen as GO fires a picture or video cue, and the
//! keys while it has the keyboard

mod editing;
mod fullscreen;
mod keys;
mod search;
mod starting;

use iced::{Size, window};

use super::Stage;
use crate::{
    back::{Cue, CueId, Kind, Picture, fresh, loaded, silent},
    front::{App, MAIN_SIZE, Message, STAGE_SIZE, view},
};

/// Returns the state of the interface with a show of `kind` cues, whose
/// files have loaded as ten seconds of picture without sound
fn show(kind: Kind) -> App {
    let cue = Cue::new(CueId(1), "1", kind, "Cue")
        .file("movie.mp4")
        .build();
    let loaded = loaded(1, "movie.mp4", silent(10.0));
    let workspace = fresh(vec![cue]).apply(loaded);
    App {
        workspace,
        main: Some(window::Id::unique()),
        ..App::default()
    }
}

/// Returns the state with a show of video cues and its stage window open
fn opened() -> App {
    show(Kind::Video).staged().0
}

/// Returns the stage window of `app`
fn stage_of(app: &App) -> window::Id {
    app.stage.window().expect("the stage window opens")
}

/// Returns the state after the operator closes the stage window
fn closing(app: App) -> App {
    let stage = stage_of(&app);
    app.closed(stage).0
}

fn picture() -> Picture {
    Picture {
        width: 1,
        height: 1,
        pixels: vec![0, 0, 0, 255],
    }
}

/// Returns the state after the stage window takes the keyboard, or else the main
/// window
fn focusing(app: App, stage: bool) -> App {
    let window = if stage { app.stage.window() } else { app.main };
    app.handled(Message::WindowFocused(window.expect("a window opens")))
        .0
}

#[test]
fn stage_window_opens_once_the_show_holds_a_picture_or_video_cue() {
    let (audio, opening) = show(Kind::Audio).staged();
    assert_eq!((audio.stage.window(), opening.units()), (None, 0));
    let (picture, opening) = show(Kind::Picture).staged();
    assert!(picture.stage.window().is_some());
    assert_eq!(opening.units(), 1);
    let (video, opening) = show(Kind::Video).staged();
    assert!(video.stage.window().is_some());
    assert_eq!(opening.units(), 1);
    let (again, opening) = video.staged();
    assert!(again.stage.window().is_some());
    assert_eq!(opening.units(), 0);
}

#[test]
fn closed_stage_window_stays_closed_until_a_video_cue_fires() {
    let open = opened();
    let stage = stage_of(&open);
    let (closed, quitting) = open.closed(stage);
    assert_eq!(quitting.units(), 0);
    let (closed, opening) = closed.staged();
    assert_eq!((closed.stage.window(), opening.units()), (None, 0));
    let (gone, _) = closed.handled(Message::Go);
    let (reopened, opening) = gone.staged();
    assert!(reopened.stage.window().is_some());
    assert_eq!(opening.units(), 1);
}

#[test]
fn stage_window_goes_fullscreen_as_go_fires_a_picture_or_video_cue() {
    let (open, _) = show(Kind::Picture).staged();
    assert!(!open.stage.settings().fullscreen);
    let (gone, _) = open.handled(Message::Go);
    // Fullscreen unless it is already, and before the other windows
    let (shown, raising) = gone.staged();
    assert_eq!(raising.units(), 2);
    let (_, raising) = shown.staged();
    assert_eq!(raising.units(), 0);
}

#[test]
fn stage_window_that_a_cue_opens_again_opens_fullscreen() {
    let (gone, _) = closing(opened()).handled(Message::Go);
    assert!(gone.stage.settings().fullscreen);
    let (reopened, _) = gone.staged();
    assert!(!reopened.stage.settings().fullscreen);
}

#[test]
fn stage_window_stays_as_it_is_when_go_fires_no_picture_or_video_cue() {
    let (gone, _) = opened().handled(Message::Go);
    let (shown, _) = gone.staged();
    // Standby has moved on past the only cue, so that GO fires nothing
    let (gone, _) = shown.handled(Message::Go);
    let (_, raising) = gone.staged();
    assert_eq!(raising.units(), 0);
}

#[test]
fn closing_the_main_window_quits() {
    let app = show(Kind::Video);
    let main = app.main.expect("the main window opens");
    let (_, quitting) = app.closed(main);
    assert_eq!(quitting.units(), 1);
}

#[test]
fn stage_keeps_the_newest_picture_of_each_video_cue_until_it_stops() {
    let (playing, _) = show(Kind::Video).handled(Message::Go);
    let stage = Stage::default().with_pictures(&playing.workspace, vec![(CueId(1), picture())]);
    assert!(stage.picture(CueId(1)).is_some());
    let kept = stage.with_pictures(&playing.workspace, Vec::new());
    assert!(kept.picture(CueId(1)).is_some());
    let (stopped, _) = playing.handled(Message::Stop(CueId(1)));
    let gone = kept.with_pictures(&stopped.workspace, Vec::new());
    assert!(gone.picture(CueId(1)).is_none());
}

#[test]
fn stage_draws_with_and_without_pictures() {
    let (playing, _) = show(Kind::Video).handled(Message::Go);
    let _ = view::stage::view(&playing);
    let stage = Stage::default().with_pictures(&playing.workspace, vec![(CueId(1), picture())]);
    let _ = view::stage::view(&App { stage, ..playing });
}

#[test]
fn stage_window_changing_size_leaves_the_cue_list_as_it_is() {
    let open = App {
        selected: Some(CueId(1)),
        ..opened()
    };
    let stage = stage_of(&open);
    let main = open.main.expect("the main window opens");
    let (open, unmoved) = open.handled(Message::Resized(stage, STAGE_SIZE));
    assert_eq!(unmoved.units(), 0);
    let (_, revealing) = open.handled(Message::Resized(main, MAIN_SIZE));
    assert_eq!(revealing.units(), 2);
}

#[test]
fn stage_window_minimized_and_then_closed_is_no_longer_drawn() {
    let open = opened();
    let stage = stage_of(&open);
    let main = open.main.expect("the main window opens");
    assert_eq!(open.drawn_windows(), [main, stage]);
    let (minimized, _) = open.handled(Message::Resized(stage, Size::ZERO));
    assert_eq!(minimized.drawn_windows(), [main]);
    let (closed, _) = minimized.handled(Message::Closed(stage));
    assert!(closed.minimized.is_empty());
    assert_eq!(closed.drawn_windows(), [main]);
}
