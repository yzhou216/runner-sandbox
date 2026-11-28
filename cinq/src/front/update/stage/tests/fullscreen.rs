// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of turning the stage window fullscreen and back with the key

use iced::window::{self, Mode};

use super::{super::lifecycle::toggled, App, Kind, Message, closing, opened, show};

#[test]
fn leaving_fullscreen_asks_the_main_window_for_the_keyboard() {
    let (stage, main) = (window::Id::unique(), window::Id::unique());
    assert_eq!(toggled(stage, Some(main), Mode::Fullscreen).units(), 2);
}

#[test]
fn entering_fullscreen_leaves_the_keyboard_alone() {
    let (stage, main) = (window::Id::unique(), window::Id::unique());
    assert_eq!(toggled(stage, Some(main), Mode::Windowed).units(), 1);
}

#[test]
fn leaving_fullscreen_with_no_main_window_only_switches_the_mode() {
    assert_eq!(
        toggled(window::Id::unique(), None, Mode::Fullscreen).units(),
        1
    );
}

#[test]
fn key_of_fullscreen_opens_the_closed_stage_window_fullscreen() {
    let (reopening, opening) = closing(opened()).toggled_fullscreen();
    assert!(reopening.stage.window().is_some());
    assert_eq!(opening.units(), 1);
    assert!(!reopening.stage.closed);
}

#[test]
fn key_of_fullscreen_opens_no_window_where_no_cue_shows_on_it() {
    let (audio, opening) = show(Kind::Audio).toggled_fullscreen();
    assert_eq!((audio.stage.window(), opening.units()), (None, 0));
    let emptied = App {
        workspace: show(Kind::Audio).workspace,
        ..closing(opened())
    };
    let (emptied, opening) = emptied.toggled_fullscreen();
    assert_eq!((emptied.stage.window(), opening.units()), (None, 0));
    assert!(!emptied.stage.cued);
}

#[test]
fn closed_stage_window_is_missing_only_while_a_cue_shows_on_it() {
    let (gone, _) = opened().handled(Message::Go);
    assert!(!gone.stage.missing(&gone.workspace));
    let closed = closing(gone);
    assert!(closed.stage.missing(&closed.workspace));
    let (reopened, _) = closed.toggled_fullscreen();
    assert!(!reopened.stage.missing(&reopened.workspace));
    let closed = closing(opened());
    assert!(!closed.stage.missing(&closed.workspace));
}
