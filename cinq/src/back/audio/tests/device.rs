// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of opening an audio device by its ID, which need no audio device,
//! since no host of cpal is named cinq

use crate::back::Audio;

#[test]
fn output_on_a_device_that_is_not_there_tells_why_nothing_is_heard() {
    let audio = Audio::open(Some("cinq:none"));
    assert_eq!(audio.error(), Some("there is no audio device cinq:none"));
}

#[test]
fn routing_to_another_device_opens_it() {
    let mut audio = Audio::open(Some("cinq:first"));
    audio.route(Some("cinq:second"));
    assert_eq!(audio.error(), Some("there is no audio device cinq:second"));
}
