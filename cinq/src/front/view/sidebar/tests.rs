// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the sidebar: the name of a list, the entry of a running cue, its
//! width, and the warnings that it lists

use unicode_width::UnicodeWidthStr;

use super::{
    LEAST, SIDEBAR,
    devices::{missing, placeholder},
    label,
    problems::limited,
    running::{entry, idle},
    width,
};
use crate::{
    back::{Command, Cue, CueId, Fault, Kind, OutputId, Workspace, fresh, loaded, sound},
    front::{App, Problems},
};

#[test]
fn a_long_list_name_gives_way_to_the_count_of_its_cues() {
    assert_eq!(label("Main", 12, 20), "Main (12)");
    assert_eq!(label("Act one, scene two", 7, 12), "Act one\u{2026} (7)");
    assert!(label(&"x".repeat(60), 120, 16).ends_with(" (120)"));
}

/// Returns a list of the looping sound Rain, which has run for a second
fn raining() -> Workspace {
    let cue = Cue::new(CueId(1), "1", Kind::Audio, "Rain")
        .file("rain.ogg")
        .looping(true)
        .build();
    fresh(vec![cue])
        .apply(loaded(1, "rain.ogg", sound(10.0)))
        .apply(Command::Go { list: 0 })
        .apply(Command::Tick(1.0))
}

#[test]
fn a_running_cue_keeps_three_lines_and_drops_whole_parts_from_the_end() {
    let running = raining().apply(Command::Panic).apply(Command::Tick(0.1));
    let active = &running.active()[0];
    let cue = running.cue(CueId(1)).expect("the cue is there");
    assert_eq!(entry(active, cue, None, 40, false, false).0, "1 Rain");
    let (_, whole) = entry(active, cue, None, 60, false, false);
    assert!(whole.starts_with("looping, ") && whole.ends_with(" dB, fading"));
    let (_, how) = entry(active, cue, None, 20, false, false);
    assert!(
        how.starts_with("looping, ") && how.ends_with(" dB"),
        "{how}"
    );
    let (_, how) = entry(active, cue, None, 14, false, false);
    assert_eq!(how, "looping");
    let (_, how) = entry(active, cue, None, 5, false, false);
    assert!(how.width() <= 5 && how.ends_with('\u{2026}'));
}

#[test]
fn the_sidebar_takes_a_quarter_of_the_window_between_its_least_and_its_most() {
    assert_eq!(width(512.0), LEAST);
    assert_eq!(width(1024.0), 256.0);
    assert_eq!(width(4000.0), SIDEBAR);
}

#[test]
fn the_sidebar_lists_three_warnings_and_counts_the_rest() {
    assert_eq!(limited(vec![1, 2, 3]), (vec![1, 2, 3], 0));
    assert_eq!(
        limited(vec![1, 2, 3, 4]),
        (vec![1, 2, 3, 4], 0),
        "a line for one warning takes the room of the warning"
    );
    assert_eq!(limited(vec![1, 2, 3, 4, 5]), (vec![1, 2, 3], 2));
    assert_eq!(limited(vec![1, 2, 3, 4, 5, 6, 7, 8]), (vec![1, 2, 3], 5));
    assert_eq!(limited(Vec::<u8>::new()), (Vec::new(), 0));
}

#[test]
fn a_running_cue_whose_outputs_all_failed_says_so_in_place_of_how_it_runs() {
    let running = raining();
    let active = &running.active()[0];
    let cue = running.cue(CueId(1)).expect("the cue is there");
    assert_eq!(
        entry(active, cue, None, 40, true, false).1,
        "silent: output failed"
    );
    assert_eq!(entry(active, cue, None, 12, true, false).1.width(), 12);
}

#[test]
fn the_default_device_list_is_lost_while_an_output_on_the_default_fails() {
    let app = App::default();
    assert!(!missing(&app));
    let failed = App {
        problems: Problems {
            audio: vec![(OutputId(1), Fault::Missing("gone".to_owned()))],
            ..Problems::default()
        },
        ..App::default()
    };
    assert!(missing(&failed), "Main is on the default device");
}

#[test]
fn a_device_that_is_not_found_reads_so_in_words_before_its_id() {
    assert_eq!(placeholder("alsa:front", true), "Not found: alsa:front");
    assert_eq!(placeholder("alsa:front", false), "alsa:front");
}

/// Returns the sound Rain and a fade of five seconds that targets it
fn rain_and_fade() -> (Cue, Cue) {
    let rain = Cue::new(CueId(1), "1", Kind::Audio, "Rain")
        .file("rain.ogg")
        .build();
    let fade = Cue::new(CueId(2), "2", Kind::Fade, "Duck")
        .target(CueId(1))
        .duration(5.0)
        .build();
    (rain, fade)
}

#[test]
fn a_running_fade_names_its_target_where_the_name_fits_whole() {
    let (rain, fade) = rain_and_fade();
    let running = fresh(vec![rain.clone(), fade])
        .apply(Command::Go { list: 0 })
        .apply(Command::Go { list: 0 })
        .apply(Command::Tick(1.0));
    let active = running.active_of(CueId(2)).expect("the fade runs");
    let cue = running.cue(CueId(2)).expect("the cue is there");
    let how = |target, columns| entry(active, cue, target, columns, false, false).1;
    assert_eq!(how(Some(&rain), 40), "0:04.0 left, fades 1");
    assert_eq!(how(Some(&rain), 12), "0:04.0 left");
    assert_eq!(how(None, 40), "0:04.0 left");
}

#[test]
fn a_fade_of_a_target_that_is_not_playing_says_so() {
    let (rain, fade) = rain_and_fade();
    let alone = fresh(vec![rain.clone(), fade])
        .apply(Command::Standby {
            list: 0,
            cue: Some(CueId(2)),
        })
        .apply(Command::Go { list: 0 })
        .apply(Command::Tick(1.0));
    let active = alone.active_of(CueId(2)).expect("the fade runs");
    let cue = alone.cue(CueId(2)).expect("the cue is there");
    assert!(idle(&alone, active, cue));
    assert_eq!(
        entry(active, cue, Some(&rain), 40, false, true).1,
        "target not playing"
    );
}
