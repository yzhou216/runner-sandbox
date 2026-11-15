// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the labels of the audio devices

use super::options;

#[test]
fn devices_go_by_name_and_by_id_where_they_share_a_name() {
    let device = |id: &str, name: &str| (id.to_owned(), name.to_owned());
    let devices = [
        device("alsa:pipewire", "PipeWire Sound Server"),
        device("alsa:hw:CARD=1,DEV=0", "USB Audio"),
        device("alsa:plughw:CARD=1,DEV=0", "USB Audio"),
        device("alsa:house", ""),
    ];
    let labels: Vec<String> = options(&devices)
        .into_iter()
        .map(|option| option.label)
        .collect();
    assert_eq!(
        labels,
        [
            "Default device",
            "PipeWire Sound Server",
            "USB Audio (alsa:hw:CARD=1,DEV=0)",
            "USB Audio (alsa:plughw:CARD=1,DEV=0)",
            "alsa:house",
        ]
    );
}
