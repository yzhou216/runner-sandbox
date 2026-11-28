// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the help beside the device list

use super::device;
use crate::{
    back::{Command, Device, Fault, OutputEdit, OutputId},
    front::{
        App, Problems,
        config::Config,
        fixtures::{main_of, two_outputs},
    },
};

/// Returns the interface of `two_outputs`, whose Main cannot play for `fault`
fn failing(fault: Fault) -> App {
    App {
        problems: Problems {
            audio: vec![(OutputId(1), fault)],
            ..Problems::default()
        },
        ..two_outputs()
    }
}

#[test]
fn a_device_that_is_not_there_gets_help_and_one_that_is_gets_none() {
    let missing = failing(Fault::Missing("not found".to_owned()));
    assert_eq!(
        device::hint(&missing, &main_of(&missing)),
        "Connect it, or choose another Default device in the sidebar"
    );
    let channels = failing(Fault::Channels(2));
    assert_eq!(device::hint(&channels, &main_of(&channels)), "");
    assert_eq!(device::hint(&two_outputs(), &main_of(&two_outputs())), "");
}

#[test]
fn a_missing_default_device_keeps_its_name_and_points_to_the_sidebar() {
    let missing = |device: &str| App {
        audio_devices: vec![(
            "alsa:pipewire".to_owned(),
            "PipeWire Sound Server".to_owned(),
        )],
        config: Config::default().with_audio_device(Some(device.to_owned())),
        ..failing(Fault::Missing("not found".to_owned()))
    };
    let gone = missing("alsa:gone");
    assert_eq!(
        device::placeholder(&gone, &main_of(&gone)),
        "Default device (alsa:gone), not found"
    );
    // Device that is there needs neither, and a named device keeps its own
    let there = App {
        problems: Problems::default(),
        ..missing("alsa:pipewire")
    };
    assert_eq!(
        device::placeholder(&there, &main_of(&there)),
        "Default device (PipeWire Sound Server)"
    );
    let named = App {
        workspace: gone.workspace.clone().apply(Command::EditOutput(
            OutputId(1),
            OutputEdit::Device(Device::Named {
                id: "alsa:usb".to_owned(),
                name: "USB Audio".to_owned(),
            }),
        )),
        ..gone
    };
    assert_eq!(
        device::placeholder(&named, &main_of(&named)),
        "USB Audio, not found"
    );
    assert_eq!(
        device::hint(&named, &main_of(&named)),
        "Connect the device, or choose another"
    );
}
