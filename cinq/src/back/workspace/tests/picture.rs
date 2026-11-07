// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of picture cues: they show until something stops them, so that
//! no cue follows them, fades and panics fade them out as they fade the
//! pictures of video cues, and show files hold them with their file alone

use std::path::Path;

use super::*;

/// Returns a picture cue that shows `slide.png`
fn picture(id: u32) -> Cue {
    Cue::new(CueId(id), id.to_string(), Kind::Picture, "Slide")
        .file("slide.png")
        .build()
}

/// Returns a workspace with one list of `cues`, whose picture files have
/// loaded as pictures, and the rest as ten seconds of sound
fn showing(cues: Vec<Cue>) -> Workspace {
    let pictures: Vec<Command> = cues
        .iter()
        .filter(|cue| cue.kind == Kind::Picture)
        .map(|cue| {
            loaded(
                cue.id.0,
                &cue.file,
                Media::Loaded {
                    duration: 0.0,
                    sound: None,
                },
            )
        })
        .collect();
    pictures.into_iter().fold(workspace(cues), Workspace::apply)
}

/// Returns how opaque the picture of cue `id` is
fn opacity(workspace: &Workspace, id: u32) -> f32 {
    workspace
        .active_of(CueId(id))
        .expect("the cue runs")
        .opacity()
}

#[test]
fn picture_cue_shows_until_something_stops_it() {
    let shown = showing(vec![picture(1)])
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
    let following = Cue {
        advance: Advance::Follow,
        ..picture(1)
    };
    let shown = showing(vec![following, audio(2)])
        .apply(GO)
        .apply(Command::Tick(60.0));
    assert!(shown.active_of(CueId(2)).is_none());
    assert_eq!(standby(&shown), Some(2));
}

#[test]
fn picture_fades_to_black_with_a_fade_that_stops_its_cue() {
    let continuing = Cue {
        advance: Advance::Continue,
        ..picture(1)
    };
    let fading = showing(vec![continuing, fade_out(2)])
        .apply(GO)
        .apply(Command::Tick(1.0));
    assert!((opacity(&fading, 1) - 0.75).abs() < 1e-6);
    assert!(fading.apply(Command::Tick(3.0)).active().is_empty());
}

#[test]
fn panic_fades_a_picture_out() {
    let panicked = showing(vec![picture(1)])
        .apply(GO)
        .apply(Command::Panic)
        .apply(Command::Tick(PANIC_FADE / 4.0));
    assert!((opacity(&panicked, 1) - 0.75).abs() < 1e-6);
    assert!(
        panicked
            .apply(Command::Tick(PANIC_FADE))
            .active()
            .is_empty()
    );
}

#[test]
fn picture_cue_plays_once_its_file_has_loaded() {
    let unnamed = Cue {
        file: String::new(),
        ..picture(1)
    };
    let unloaded = Workspace::new(vec![CueList::new("Main", vec![picture(1), unnamed])]);
    let problems: Vec<Problem> = unloaded.problems().map(|(_, problem)| problem).collect();
    assert_eq!(problems, [Problem::NotLoaded, Problem::NoFile]);
    assert_eq!(showing(vec![picture(1)]).problems().count(), 0);
}

#[test]
fn new_fade_cue_targets_the_picture_cue_it_follows() {
    let workspace = showing(vec![picture(1)]).apply(add(0, Kind::Fade, Place::After(CueId(1))));
    assert_eq!(workspace.lists()[0].cues[1].target, Some(CueId(1)));
}

#[test]
fn show_file_holds_a_picture_cue_with_its_file_alone() {
    let workspace = showing(vec![picture(1)]).directory("/show");
    let text = workspace.show(Some(Path::new("/show")));
    let cue = "kind = \"picture\"\nname = \"Slide\"\nfile = \"slide.png\"\n";
    assert!(text.ends_with(cue), "{text}");
    let read = Workspace::from_show(&text, "/show").expect("the show reads back");
    assert_eq!(read.show(Some(Path::new("/show"))), text);
}
