// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of video cues: their pictures fade to black with the fades that
//! stop them, fade cues act on them, and show files hold them as they hold
//! audio cues

use super::*;

fn video(id: u32) -> Cue {
    Cue::new(CueId(id), id.to_string(), Kind::Video, "Movie")
        .file("movie.mp4")
        .build()
}

#[test]
fn picture_fades_to_black_with_a_fade_that_stops_its_cue() {
    let workspace = workspace(vec![continued(video(1)), fade_out(2)])
        .apply(GO)
        .apply(Command::Tick(1.0));
    assert!(workspace.active_of(CueId(1)).unwrap().shows_picture());
    assert!((opacity(&workspace, 1) - 0.75).abs() < 1e-6);
    assert!(workspace.apply(Command::Tick(3.0)).active().is_empty());
}

#[test]
fn picture_fading_out_fades_on_from_where_it_was_under_a_panic_or_another_fade() {
    let longer = Cue {
        duration: 8.0.into(),
        ..fade_out(3)
    };
    let halfway = workspace(vec![continued(video(1)), fade_out(2), longer])
        .apply(GO)
        .apply(Command::Tick(2.0));
    let panicked = halfway.clone().apply(Command::Panic);
    assert!((opacity(&panicked, 1) - 0.5).abs() < 1e-6);
    let refaded = halfway.apply(GO).apply(Command::Tick(4.0));
    assert!((opacity(&refaded, 1) - 0.25).abs() < 1e-6);
}

#[test]
fn picture_stays_whole_under_a_fade_that_leaves_its_cue_playing() {
    let workspace = workspace(vec![continued(video(1)), fade_under(2)])
        .apply(GO)
        .apply(Command::Tick(2.0));
    assert_eq!(opacity(&workspace, 1), 1.0);
}

#[test]
fn panic_fades_the_picture_out_with_the_sound() {
    let panicked = workspace(vec![video(1)])
        .apply(GO)
        .apply(Command::Panic)
        .apply(Command::Tick(PANIC_FADE / 4.0));
    assert!((opacity(&panicked, 1) - 0.75).abs() < 1e-6);
    let level = level(&panicked, 1).expect("the cue runs");
    assert!((level - SILENCE / 4.0).abs() < 1e-4);
}

#[test]
fn video_cue_without_sound_still_fades_out_and_stops() {
    let workspace = workspace(vec![continued(video(1)), fade_out(2)])
        .apply(loaded(1, "movie.mp4", silent(10.0)))
        .apply(GO);
    assert_eq!(workspace.active_of(CueId(1)).unwrap().sound(), None);
    let halfway = workspace.apply(Command::Tick(2.0));
    assert!((opacity(&halfway, 1) - 0.5).abs() < 1e-6);
    assert!(halfway.apply(Command::Tick(2.0)).active().is_empty());
}

#[test]
fn new_fade_cue_targets_the_video_cue_it_follows() {
    let workspace = workspace(vec![video(1)]).apply(add(0, Kind::Fade, Place::After(CueId(1))));
    assert_eq!(workspace.lists()[0].cues[1].target, Some(CueId(1)));
}

#[test]
fn show_file_holds_a_video_cue_as_it_holds_an_audio_cue() {
    let looping = Cue {
        start: 1.5.into(),
        looping: true,
        ..video(1)
    };
    let text = shown(&workspace(vec![looping]).directory("/show"));
    let cue = "kind = \"video\"\nname = \"Movie\"\nfile = \"movie.mp4\"\nstart = 1.5\nloop = true\nlevel = 0.0\n";
    assert!(text.contains(cue), "{text}");
    read_back(&text);
}

#[test]
fn clock_of_video_run_moves_a_tenth_of_the_way_to_the_time_its_sound_has_reached() {
    let ticked = workspace(vec![video(1)])
        .apply(GO)
        .apply(Command::Tick(1.0));
    let run = ticked.active_of(CueId(1)).unwrap().run;
    let heard = |workspace: &Workspace, run, played| {
        let heard = workspace.clone().apply(Command::Heard {
            cue: CueId(1),
            run,
            played,
        });
        heard.active_of(CueId(1)).unwrap().played()
    };
    assert!((heard(&ticked, run, 1.2) - 1.02).abs() < 1e-9);
    assert!((heard(&ticked, run, 0.8) - 0.98).abs() < 1e-9);
    assert_eq!(heard(&ticked, run + 1, 1.2), 1.0);
    assert_eq!(heard(&ticked.apply(Command::Pause(true)), run, 1.2), 1.0);
}
