// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the warnings about outputs that cannot play

use super::{Warning, warnings};
use crate::{
    back::{Command, CueList, Device, Fault, OutputEdit, OutputId, Workspace},
    front::App,
};

/// Returns an app whose show has an output on each of `devices`, the first
/// of them Main, which cannot play for `faults`
fn app(devices: &[Device], faults: Vec<(OutputId, Fault)>) -> App {
    let workspace = devices.iter().enumerate().fold(
        Workspace::new(vec![CueList::new("Main", Vec::new())]),
        |workspace, (index, device)| {
            let added = if index == 0 {
                workspace
            } else {
                workspace.apply(Command::AddOutput { at: index })
            };
            let id = added.outputs()[index].id;
            added.apply(Command::EditOutput(id, OutputEdit::Device(device.clone())))
        },
    );
    App {
        workspace,
        audio_problems: faults,
        ..App::default()
    }
}

fn dante() -> Device {
    Device::Named {
        id: "coreaudio:DanteVirtualSoundcard_UID".to_owned(),
        name: "Dante Virtual Soundcard".to_owned(),
    }
}

fn missing() -> Fault {
    Fault::Missing("there is no audio device Dante Virtual Soundcard".to_owned())
}

#[test]
fn outputs_of_a_missing_device_share_one_warning() {
    let app = app(
        &[dante(), Device::Default, dante(), dante()],
        vec![
            (OutputId(1), missing()),
            (OutputId(3), missing()),
            (OutputId(4), missing()),
        ],
    );
    let device = dante();
    let fault = missing();
    assert_eq!(
        warnings(&app),
        [Warning::Device {
            device: &device,
            fault: &fault,
            outputs: vec![OutputId(1), OutputId(3), OutputId(4)],
        }]
    );
}

#[test]
fn output_past_the_channels_of_its_device_warns_alone() {
    let app = app(
        &[dante(), dante()],
        vec![(OutputId(2), Fault::Channels(16))],
    );
    let found = warnings(&app);
    assert_eq!(found.len(), 1);
    assert!(matches!(found[0], Warning::Channels { output, .. } if output.id == OutputId(2)));
    assert_eq!(found[0].output(), OutputId(2));
}

#[test]
fn outputs_that_play_give_no_warning() {
    assert_eq!(warnings(&app(&[dante(), Device::Default], Vec::new())), []);
}
