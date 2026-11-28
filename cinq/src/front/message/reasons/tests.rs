// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the reasons that a setting of an output or of the light patch
//! does not apply

use super::{channels, channels_help, level, name, rejection};
use crate::{
    back::{CueId, InstrumentId, OutputId},
    front::{
        Setting,
        fixtures::{main_of, two_outputs},
    },
};

#[test]
fn a_name_that_is_empty_or_that_another_output_has_is_rejected_with_its_reason() {
    let app = two_outputs();
    let main = main_of(&app);
    let rejected = |text| name(&app, &main, text);
    assert_eq!(rejected("Main"), None, "its own name");
    assert_eq!(rejected("Rear"), None);
    assert_eq!(rejected("").as_deref(), Some("the name is empty"));
    assert_eq!(rejected("  ").as_deref(), Some("the name is empty"));
    assert_eq!(
        rejected("Output 1").as_deref(),
        Some("another output has this name")
    );
}

#[test]
fn channels_that_are_neither_one_nor_two_are_rejected_and_the_help_names_what_a_channel_is() {
    ["3", "1-2", "1,3", "256"]
        .into_iter()
        .for_each(|text| assert_eq!(channels(text), None, "{text}"));
    ["", "1-23-4", "1-3", "0", "257", "-2"]
        .into_iter()
        .for_each(|text| {
            assert_eq!(
                channels(text).as_deref(),
                Some("channels are 3 for one, or 1-2 or 1,3 for two, up to 256"),
                "{text:?}"
            );
        });
    assert_eq!(
        channels_help(),
        "Channels 1 to 256: 3 for one, 1-2 or 1,3 for two"
    );
}

#[test]
fn a_level_out_of_range_is_rejected_but_an_empty_one_sends_nothing() {
    ["", "  ", "+0.0", "-6 dB", "12", "-60"]
        .into_iter()
        .for_each(|text| assert_eq!(level(text), None, "{text:?}"));
    ["loud", "13", "-61"].into_iter().for_each(|text| {
        assert_eq!(
            level(text).as_deref(),
            Some("levels run from -60 to +12 dB, or empty for none"),
            "{text:?}"
        );
    });
}

#[test]
fn a_light_level_or_an_address_out_of_range_is_rejected_with_its_reason() {
    let app = two_outputs();
    let reason = |setting, text| rejection(&app, setting, text);
    let level = Setting::Light(CueId(1), InstrumentId(1));
    ["", "0", "100", "12.5%"]
        .into_iter()
        .for_each(|text| assert_eq!(reason(level, text), None, "{text:?}"));
    ["101", "-1", "bright"].into_iter().for_each(|text| {
        assert_eq!(
            reason(level, text).as_deref(),
            Some("0 to 100, or empty for unchanged"),
            "{text:?}"
        );
    });
    let address = Setting::Address(InstrumentId(1));
    ["1", "512", "2/12", "32768/512"]
        .into_iter()
        .for_each(|text| assert_eq!(reason(address, text), None, "{text:?}"));
    // The range comes first, so that the status bar keeps it where it cuts
    ["0", "513", "0/1", "32769/1", ""]
        .into_iter()
        .for_each(|text| {
            assert!(
                reason(address, text).is_some_and(|reason| reason.starts_with("1 to 512")),
                "{text:?}"
            );
        });
}

#[test]
fn a_send_level_out_of_range_says_that_empty_takes_the_send_away() {
    let app = two_outputs();
    assert_eq!(
        rejection(&app, Setting::Send(CueId(1), OutputId(1)), ""),
        None
    );
    assert_eq!(
        rejection(&app, Setting::Send(CueId(1), OutputId(1)), "loud").as_deref(),
        Some("levels run from -60 to +12 dB, or empty for no send")
    );
}
