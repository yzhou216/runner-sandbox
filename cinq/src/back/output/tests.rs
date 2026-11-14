// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the channels of outputs, as people type them

use super::Channels;

#[test]
fn channels_read_as_people_count_them_and_write_back_the_same() {
    let cases = [
        ("3", Some(Channels::Mono(2))),
        ("1-2", Some(Channels::Stereo(0, 1))),
        (" 5 - 6 ", Some(Channels::Stereo(4, 5))),
        ("1,3", Some(Channels::Stereo(0, 2))),
        ("64", Some(Channels::Mono(63))),
        ("127-128", Some(Channels::Stereo(126, 127))),
        ("256", Some(Channels::Mono(255))),
        // Range of more than two channels, no channel, or one past the
        // channels that a device can have
        ("1-3", None),
        ("2-1", None),
        ("2,2", None),
        ("0", None),
        ("257", None),
        ("", None),
        ("left", None),
    ];
    cases.into_iter().for_each(|(text, channels)| {
        assert_eq!(Channels::parse(text), channels, "{text:?}");
    });
    ["3", "1-2", "1,3"].into_iter().for_each(|text| {
        let written = Channels::parse(text).map(|channels| channels.to_string());
        assert_eq!(written.as_deref(), Some(text));
    });
    assert_eq!(Channels::Stereo(4, 5).needs(), 6);
}
