// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of picture cues: they show until something stops them, so that
//! no cue follows them, fades and panics fade them out as they fade the
//! pictures of video cues, and show files hold them with their file alone

use super::*;

#[test]
fn picture_cue_shows_until_something_stops_it() {
    let shown = workspace(vec![picture(1)])
        .apply(GO)
        .apply(Command::Tick(3600.0));
    let active = shown.active_of(CueId(1)).expect("the picture still shows");
    assert!(active.shows_picture());
    assert!(!active.shows_video());
    assert_eq!(active.phase(), Phase::Showing);
    assert!(shown.apply(Command::Stop(CueId(1))).active().is_empty());
}

#[test]
fn cue_after_a_picture_cue_that_follows_waits_for_go() {
    let shown = workspace(vec![following(picture(1)), audio(2)])
        .apply(GO)
        .apply(Command::Tick(60.0));
    assert!(shown.active_of(CueId(2)).is_none());
    assert_eq!(standby(&shown), Some(2));
}

#[test]
fn picture_and_video_fade_to_black_with_a_fade_that_stops_their_cue() {
    [picture(1), video(1)].into_iter().for_each(|cue| {
        let kind = cue.kind;
        let fading = workspace(vec![continued(cue), fade_out(2)])
            .apply(GO)
            .apply(Command::Tick(1.0));
        assert!(
            fading.active_of(CueId(1)).unwrap().shows_picture(),
            "{kind:?}"
        );
        assert!((opacity(&fading, 1) - 0.75).abs() < 1e-6, "{kind:?}");
        assert!(
            fading.apply(Command::Tick(3.0)).active().is_empty(),
            "{kind:?}"
        );
    });
}

#[test]
fn panic_fades_a_picture_and_a_video_out_with_their_sound() {
    [picture(1), video(1)].into_iter().for_each(|cue| {
        let kind = cue.kind;
        let panicked = workspace(vec![cue])
            .apply(GO)
            .apply(Command::Panic)
            .apply(Command::Tick(PANIC_FADE / 4.0));
        assert!((opacity(&panicked, 1) - 0.75).abs() < 1e-6, "{kind:?}");
        let level = level(&panicked, 1).expect("the cue runs");
        assert!((level - SILENCE / 4.0).abs() < 1e-4, "{kind:?}");
        let ended = panicked.apply(Command::Tick(PANIC_FADE));
        assert!(ended.active().is_empty(), "{kind:?}");
    });
}

#[test]
fn picture_cue_plays_once_its_file_has_loaded() {
    let unnamed = Cue {
        file: String::new(),
        ..picture(2)
    };
    let unloaded = fresh(vec![picture(1), unnamed]);
    assert_eq!(
        problems(&unloaded),
        [(1, Problem::NotLoaded), (2, Problem::NoFile)]
    );
    assert_eq!(workspace(vec![picture(1)]).problems().count(), 0);
}

#[test]
fn show_file_holds_a_picture_cue_with_its_file_alone() {
    let text = shown(&workspace(vec![picture(1)]).directory("/show"));
    let cue = "kind = \"picture\"\nname = \"Slide\"\nfile = \"slide.png\"\n";
    assert!(text.ends_with(cue), "{text}");
}
