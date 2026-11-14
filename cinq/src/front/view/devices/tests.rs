// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the labels of the audio devices

use super::{choices, chosen};
use crate::back::Device;

#[test]
fn devices_go_by_name_and_by_id_where_they_share_a_name() {
    let device = |id: &str, name: &str| (id.to_owned(), name.to_owned());
    let devices = [
        device("alsa:pipewire", "PipeWire Sound Server"),
        device("alsa:hw:CARD=1,DEV=0", "USB Audio"),
        device("alsa:plughw:CARD=1,DEV=0", "USB Audio"),
        device("alsa:house", ""),
    ];
    let labels: Vec<String> = choices(&devices, "System default")
        .into_iter()
        .map(|option| option.label)
        .collect();
    assert_eq!(
        labels,
        [
            "System default",
            "PipeWire Sound Server",
            "USB Audio (alsa:hw:CARD=1,DEV=0)",
            "USB Audio (alsa:plughw:CARD=1,DEV=0)",
            "alsa:house",
        ]
    );
}

#[test]
fn output_chooses_its_device_by_id_or_else_by_its_only_name() {
    let device = |id: &str, name: &str| (id.to_owned(), name.to_owned());
    let devices = [
        device("wasapi:{one}", "Speakers (VB-Audio Virtual Cable)"),
        device("alsa:hw:CARD=1,DEV=0", "USB Audio"),
        device("alsa:plughw:CARD=1,DEV=0", "USB Audio"),
    ];
    let options = choices(&devices, "Default device");
    let named = |id: &str, name: &str| Device::Named {
        id: id.to_owned(),
        name: name.to_owned(),
    };
    let id = |device: &Device| chosen(&options, device).and_then(|option| option.id.clone());
    assert_eq!(
        chosen(&options, &Device::Default).map(|option| option.id.clone()),
        Some(None)
    );
    assert_eq!(
        id(&named("wasapi:{one}", "Speakers")).as_deref(),
        Some("wasapi:{one}")
    );
    // Show made on another machine, whose ID this one lacks
    assert_eq!(
        id(&named(
            "wasapi:{other}",
            "Speakers (VB-Audio Virtual Cable)"
        ))
        .as_deref(),
        Some("wasapi:{one}")
    );
    // Name of two devices, which no list can tell apart
    assert_eq!(id(&named("alsa:hw:CARD=2,DEV=0", "USB Audio")), None);
    assert_eq!(id(&named("coreaudio:gone", "Gone")), None);
}
