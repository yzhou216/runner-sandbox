// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the clips of video and picture cues

use super::*;
use crate::back::silent;

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
        media: silent(0.0),
        advance: Advance::Follow,
        ..Cue::new(CueId(1), "1", Kind::Picture, "New Picture cue")
            .file("slide.png")
            .build()
    };
    assert_eq!(
        (picture.clip(), picture.shown()),
        (None, Some(Shown::Still))
    );
    assert!(picture.endless() && !picture.sounds());
    assert_eq!(next_delay(&picture), None);
    let named = picture.playing("slides/preshow.png".to_owned());
    assert!(matches!(named, Edit::FileAndName { name, .. } if name == "preshow"));
}
