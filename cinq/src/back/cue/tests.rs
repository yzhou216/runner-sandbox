// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of cues: their levels, names, clips, and delays

use super::*;

fn audio(name: &str, file: &str) -> Cue {
    Cue::new(CueId(1), "1", Kind::Audio, name)
        .file(file)
        .build()
}

#[test]
fn typed_level_reads_with_or_without_its_unit() {
    assert_eq!(parse_level("-6"), Some(-6.0));
    assert_eq!(parse_level(" +3.5 dB "), Some(3.5));
    assert_eq!(parse_level("-60db"), Some(-60.0));
}

#[test]
fn typed_level_past_either_end_or_half_typed_reads_as_none() {
    assert_eq!(parse_level("-"), None);
    assert_eq!(parse_level("-61"), None);
    assert_eq!(parse_level("13"), None);
    assert_eq!(parse_level("NaN"), None);
}

#[test]
fn cue_without_a_number_displays_as_its_name_alone() {
    let numbered = audio("Rain", "rain.wav");
    let unnumbered = Cue::new(CueId(2), " ", Kind::Audio, "Rain").build();
    assert_eq!(numbered.to_string(), "1 Rain");
    assert_eq!(unnumbered.to_string(), "Rain");
}

#[test]
fn chosen_sound_file_names_a_cue_until_the_operator_names_it() {
    let chosen = |cue: Cue| cue.playing("sounds/thunder.wav".to_owned());
    let thunder = Edit::FileAndName {
        file: "sounds/thunder.wav".to_owned(),
        name: "thunder".to_owned(),
    };
    assert_eq!(chosen(audio(&Cue::new_name(Kind::Audio), "")), thunder);
    assert_eq!(chosen(audio("storm", "sounds/storm.ogg")), thunder);
    assert_eq!(
        chosen(audio("Storm build", "sounds/storm.ogg")),
        Edit::File("sounds/thunder.wav".to_owned())
    );
}

fn loaded(cue: Cue) -> Cue {
    Cue {
        media: Media::Loaded {
            duration: 10.0,
            sound: Some(10.0),
        },
        ..cue
    }
}

#[test]
fn clip_runs_from_the_start_to_the_end_or_to_the_end_of_the_file() {
    let whole = loaded(audio("storm", "storm.ogg"));
    let clip = |start: f64, end: f64| {
        Some(Clip {
            start,
            end,
            looping: false,
        })
    };
    assert_eq!(whole.clip(), clip(0.0, 10.0));
    let trimmed = Cue {
        start: 1.5.into(),
        end: 4.0.into(),
        ..whole.clone()
    };
    assert_eq!(trimmed.clip(), clip(1.5, 4.0));
    assert_eq!(trimmed.action(), 2.5);
    let past = Cue {
        end: 30.0.into(),
        ..whole
    };
    assert_eq!(past.clip(), clip(0.0, 10.0));
}

#[test]
fn cue_that_starts_at_or_after_its_end_has_no_clip() {
    let whole = loaded(audio("storm", "storm.ogg"));
    let at_end = Cue {
        start: 4.0.into(),
        end: 4.0.into(),
        ..whole.clone()
    };
    assert_eq!(at_end.clip(), None);
    let past_file = Cue {
        start: 12.0.into(),
        ..whole
    };
    assert_eq!(past_file.clip(), None);
}

#[test]
fn clip_that_loops_goes_back_to_its_start_at_its_end() {
    let clip = Clip {
        start: 2.0,
        end: 5.0,
        looping: true,
    };
    assert_eq!(clip.position(1.0), 3.0);
    assert_eq!(clip.position(4.0), 3.0);
    let once = Clip {
        looping: false,
        ..clip
    };
    assert_eq!(once.position(4.0), 6.0);
}

#[test]
fn audio_cue_that_loops_follows_on_to_nothing() {
    let looping = Cue {
        looping: true,
        advance: Advance::Follow,
        ..loaded(audio("rain", "rain.ogg"))
    };
    assert_eq!(looping.next_delay(), None);
    let continuing = Cue {
        advance: Advance::Continue,
        ..looping
    };
    assert_eq!(continuing.next_delay(), Some(0.0));
}

#[test]
fn silence_is_where_kira_stops_making_sound() {
    assert_eq!(SILENCE, kira::Decibels::SILENCE.0);
}

fn video(start: f32, looping: bool) -> Cue {
    Cue {
        media: Media::Loaded {
            duration: 10.0,
            sound: Some(9.5),
        },
        ..Cue::new(CueId(1), "1", Kind::Video, "Movie")
            .file("movie.mp4")
            .start(start)
            .looping(looping)
            .build()
    }
}

#[test]
fn video_cue_that_loops_ends_with_its_sound_where_that_ends_first() {
    let end = |cue: Cue| cue.clip().map(|clip| clip.end);
    assert_eq!(end(video(0.0, true)), Some(9.5));
    assert_eq!(end(video(0.0, false)), Some(10.0));
    assert!(video(0.0, true).sounds());
}

#[test]
fn video_cue_that_starts_past_the_end_of_its_sound_plays_its_picture_alone() {
    let late = video(9.7, true);
    assert!(!late.sounds());
    let clip = late.clip().expect("the picture plays");
    assert_eq!((clip.start as f32, clip.end), (9.7, 10.0));
}

#[test]
fn picture_cue_shows_its_file_whole_and_follows_on_to_nothing() {
    let picture = Cue {
        media: Media::Loaded {
            duration: 0.0,
            sound: None,
        },
        advance: Advance::Follow,
        ..Cue::new(CueId(1), "1", Kind::Picture, "New Picture cue").build()
    };
    assert_eq!(
        (picture.clip(), picture.shown()),
        (None, Some(Shown::Still))
    );
    assert!(picture.endless() && !picture.sounds());
    assert_eq!(picture.next_delay(), None);
    let named = picture.playing("slides/preshow.png".to_owned());
    assert!(matches!(named, Edit::FileAndName { name, .. } if name == "preshow"));
}

#[test]
fn audio_cue_that_follows_goes_on_when_its_clip_ends() {
    let following = Cue {
        advance: Advance::Follow,
        pre_wait: 1.5.into(),
        post_wait: 9.0.into(),
        start: 2.0.into(),
        ..loaded(audio("rain", "rain.ogg"))
    };
    // Auto-follow ignores the post-wait
    assert_eq!(following.next_delay(), Some(9.5));
}

#[test]
fn audio_cue_that_continues_goes_on_after_its_waits() {
    let continuing = Cue {
        advance: Advance::Continue,
        pre_wait: 1.5.into(),
        post_wait: 2.0.into(),
        ..loaded(audio("rain", "rain.ogg"))
    };
    assert_eq!(continuing.next_delay(), Some(3.5));
}

#[test]
fn audio_cue_that_waits_for_go_goes_on_to_nothing() {
    assert_eq!(loaded(audio("rain", "rain.ogg")).next_delay(), None);
}

#[test]
fn clip_has_played_as_far_as_its_sound_has_reached_in_its_file() {
    let clip = |looping| Clip {
        start: 2.0,
        end: 5.0,
        looping,
    };
    assert_eq!(clip(false).played_at(4.5, 2.4), 2.5);
    // Three seconds a pass: the sound at 3.0 in the file is a second into a
    // pass, and the clock tells which
    let pass = |position, near| clip(true).played_at(position, near);
    assert_eq!(pass(3.0, 0.5), 1.0);
    assert_eq!(pass(3.0, 3.4), 4.0);
    // Sound just past the loop point, with a clock just short of it, and
    // the other way round
    assert!((pass(2.1, 2.9) - 3.1).abs() < 1e-9);
    assert!((pass(4.9, 3.1) - 2.9).abs() < 1e-9);
    // Each position of the sound maps back to the time it came from
    assert!(
        [0.0, 0.7, 3.0, 4.2, 20.5]
            .iter()
            .all(|&at| (pass(clip(true).position(at), at) - at).abs() < 1e-9)
    );
}
