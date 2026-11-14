// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of where outputs play and of the gains that the sends of a cue
//! make on each device

use crate::back::{
    Channels, Device, Output, OutputId, Sends,
    audio::patch::{Key, Need, Patch},
};

fn output(id: u32, device: Option<&str>, channels: Channels) -> Output {
    Output {
        id: OutputId(id),
        name: format!("Output {id}"),
        device: device.map_or(Device::Default, |device| Device::Named {
            id: device.to_owned(),
            name: String::new(),
        }),
        channels,
        new_cues: None,
    }
}

fn sends(pairs: &[(u32, f32)]) -> Sends {
    pairs
        .iter()
        .map(|&(id, level)| (OutputId(id), level))
        .collect()
}

/// Outputs of a desk on its own device, House on its channels 3 and 4 and
/// Sub on its channel 6, and a lobby on the default device
fn patch() -> Patch {
    Patch::of(
        &[
            output(1, Some("desk"), Channels::Stereo(2, 3)),
            output(2, Some("desk"), Channels::Mono(5)),
            output(3, None, Channels::MAIN),
        ],
        &Some("laptop".to_owned()),
    )
}

fn key(name: &str) -> Key {
    Some(name.to_owned())
}

#[test]
fn devices_open_once_with_the_channels_that_their_outputs_need() {
    let needs = patch().needs();
    let opened: Vec<(Key, u16)> = needs
        .into_iter()
        .map(|Need { key, channels, .. }| (key, channels))
        .collect();
    assert_eq!(opened, [(key("desk"), 6), (key("laptop"), 2)]);
}

#[test]
fn stereo_outputs_take_left_and_right_and_mono_ones_half_of_each() {
    let gains = patch().gains(&sends(&[(1, 0.0), (2, 0.0)]), |_| Some(8));
    assert_eq!(gains.len(), 1);
    let (device, gains) = &gains[0];
    assert_eq!(*device, key("desk"));
    assert_eq!(
        gains.as_ref(),
        [
            [0.0, 0.0],
            [0.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [0.0, 0.0],
            [0.5, 0.5],
            [0.0, 0.0],
            [0.0, 0.0],
        ]
    );
}

#[test]
fn levels_of_sends_scale_their_gains_and_devices_follow_the_order_of_outputs() {
    let gains = patch().gains(&sends(&[(3, -6.0), (2, -60.0)]), |_| Some(6));
    let devices: Vec<&Key> = gains.iter().map(|(device, _)| device).collect();
    assert_eq!(devices, [&key("desk"), &key("laptop")]);
    assert_eq!(gains[0].1[5], [0.0, 0.0], "-60 dB is silence");
    let half = gains[1].1[0][0];
    assert!((half - 0.501).abs() < 1e-3, "{half}");
}

#[test]
fn channels_past_those_of_a_device_play_nowhere() {
    let gains = patch().gains(&sends(&[(1, 0.0), (2, 0.0)]), |_| Some(4));
    assert_eq!(
        gains[0].1.as_ref(),
        [[0.0, 0.0], [0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]
    );
    let closed = patch().gains(&sends(&[(1, 0.0)]), |_| None);
    assert!(
        closed.is_empty(),
        "a device that did not open plays nothing"
    );
}
