// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the streams and the devices that the mix renders for

use std::time::Duration;

use kira::backend::cpal::cpal::{ErrorKind, SampleFormat};

#[test]
fn stream_that_has_yet_to_call_has_longer_than_one_that_has() {
    let seconds = Duration::from_secs;
    assert!(super::stalled(seconds(3), Some(Duration::ZERO)));
    assert!(!super::stalled(seconds(3), None));
    assert!(super::stalled(seconds(10), None));
    assert!(!super::stalled(seconds(11), Some(seconds(9))));
}

#[test]
fn stream_is_built_anew_after_its_device_goes_but_not_after_an_underrun() {
    assert!(super::stream::ends(ErrorKind::DeviceNotAvailable));
    assert!(super::stream::ends(ErrorKind::StreamInvalidated));
    assert!(!super::stream::ends(ErrorKind::Xrun));
    assert!(!super::stream::ends(ErrorKind::BackendError));
}

#[test]
fn device_opens_with_the_fewest_channels_that_hold_the_outputs_or_else_all_it_has() {
    use SampleFormat::{F32, I16, I32, U8};
    let chosen = super::config::chosen;
    let wide = [(F32, 2), (F32, 16), (I32, 64)];
    assert_eq!(chosen(&wide, 3), Some((F32, 16)), "the fewest that hold");
    assert_eq!(chosen(&wide, 2), Some((F32, 2)), "exactly as many");
    let narrow = [(F32, 2), (F32, 16)];
    assert_eq!(chosen(&narrow, 34), Some((F32, 16)), "all it has");
    let mixed = [(F32, 2), (I32, 8)];
    assert_eq!(
        chosen(&mixed, 34),
        Some((I32, 8)),
        "the most of any samples"
    );
    let late = [(F32, 2), (I16, 8)];
    assert_eq!(chosen(&late, 4), Some((I16, 8)), "channels before samples");
    assert_eq!(chosen(&[], 2), None, "no offers");
    assert_eq!(chosen(&[(U8, 2)], 2), None, "no samples that cinq plays");
}

#[test]
fn hw_device_of_samples_that_cinq_cannot_play_names_its_plughw_device() {
    assert_eq!(
        super::config::unplayable_reason("alsa:hw:CARD=USB,DEV=0"),
        "it takes no samples that cinq can play; choose alsa:plughw:CARD=USB,DEV=0 instead"
    );
    assert_eq!(
        super::config::unplayable_reason("asio:FlexASIO"),
        "it takes no samples that cinq can play"
    );
}
