// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of when a cue goes on to the next one

use super::*;

#[test]
fn audio_cue_that_loops_follows_on_to_nothing() {
    let looping = Cue {
        looping: true,
        advance: Advance::Follow,
        ..loaded(audio("rain", "rain.ogg"))
    };
    assert_eq!(next_delay(&looping), None);
    let continuing = Cue {
        advance: Advance::Continue,
        ..looping
    };
    assert_eq!(next_delay(&continuing), Some(0.0));
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
    assert_eq!(next_delay(&following), Some(9.5));
}

#[test]
fn audio_cue_that_continues_goes_on_after_its_waits() {
    let continuing = Cue {
        advance: Advance::Continue,
        pre_wait: 1.5.into(),
        post_wait: 2.0.into(),
        ..loaded(audio("rain", "rain.ogg"))
    };
    assert_eq!(next_delay(&continuing), Some(3.5));
}

#[test]
fn audio_cue_that_waits_for_go_goes_on_to_nothing() {
    assert_eq!(next_delay(&loaded(audio("rain", "rain.ogg"))), None);
}
