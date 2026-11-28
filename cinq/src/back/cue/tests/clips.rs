// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the clip that a cue plays and of the loop inside it

use super::*;

#[test]
fn clip_runs_from_the_start_to_the_end_or_to_the_end_of_the_file() {
    let whole = loaded(audio("storm", "storm.ogg"));
    let clip = |start: f64, end: f64| {
        Some(Clip {
            start,
            end,
            looping: None,
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
    let clip = looping_clip(true);
    assert_eq!(clip.position(1.0), 3.0);
    assert_eq!(clip.position(4.0), 3.0);
    assert_eq!(looping_clip(false).position(4.0), 6.0);
}

/// Returns a cue that plays from 1 to 8 seconds and loops between the
/// given times
fn vamp(loop_start: f32, loop_end: f32) -> Cue {
    Cue {
        start: 1.0.into(),
        end: 8.0.into(),
        looping: true,
        loop_start: loop_start.into(),
        loop_end: loop_end.into(),
        ..loaded(audio("vamp", "vamp.ogg"))
    }
}

#[test]
fn loop_lies_within_the_clip_of_its_cue() {
    let section = |cue: Cue| cue.clip().and_then(|clip| clip.looping);
    let within = |start, end| {
        Some(Loop {
            start,
            end,
            released: None,
        })
    };
    assert_eq!(section(vamp(3.0, 5.0)), within(3.0, 5.0));
    assert_eq!(section(vamp(0.0, 30.0)), within(1.0, 8.0));
    let whole = Cue {
        loop_start: Seconds::default(),
        loop_end: Seconds::default(),
        ..vamp(3.0, 5.0)
    };
    assert_eq!(section(whole), within(1.0, 8.0));
    let backward = vamp(5.0, 3.0);
    assert_eq!(backward.clip(), None);
    assert!(backward.loops_nothing());
    assert!(!vamp(3.0, 5.0).loops_nothing());
}

#[test]
fn loop_played_tells_where_the_loop_that_plays_differs_from_the_one_typed() {
    assert_eq!(vamp(3.0, 5.0).loop_played(), None);
    assert_eq!(vamp(3.0, 30.0).loop_played(), Some((3.0, 8.0)));
    assert_eq!(vamp(0.5, 5.0).loop_played(), Some((1.0, 5.0)));
    assert_eq!(vamp(5.0, 3.0).loop_played(), None);
    let whole = Cue {
        loop_start: Seconds::default(),
        loop_end: Seconds::default(),
        ..vamp(3.0, 5.0)
    };
    assert_eq!(whole.loop_played(), None);
}

#[test]
fn loop_shorter_than_a_millisecond_plays_nothing_as_kira_would_spin_on_it() {
    let brief = vamp(2.0, 2.0005);
    assert_eq!(brief.clip(), None);
    assert!(brief.loops_briefly());
    assert!(!brief.loops_nothing());
    assert_eq!(brief.loop_played(), None);
    assert!(!vamp(2.0, 2.002).loops_briefly());
    assert!(vamp(2.0, 2.002).clip().is_some());
    assert!(!vamp(5.0, 3.0).loops_briefly());
}
