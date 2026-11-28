// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the warnings about outputs that cannot play

use super::{Warning, failing, warnings};
use crate::{
    back::{Channels, Command, Device, Fault, OutputEdit, OutputId, fresh},
    front::{App, Problems},
};

/// Returns an app whose show has an output on each of `devices`, the first
/// of them Main, which cannot play for `faults`
fn app(devices: &[Device], faults: Vec<(OutputId, Fault)>) -> App {
    let workspace =
        devices
            .iter()
            .enumerate()
            .fold(fresh(Vec::new()), |workspace, (index, device)| {
                let added = if index == 0 {
                    workspace
                } else {
                    workspace.apply(Command::add_output(index, 1))
                };
                let id = added.outputs()[index].id;
                added.apply(Command::EditOutput(id, OutputEdit::Device(device.clone())))
            });
    App {
        workspace,
        problems: Problems {
            audio: faults,
            ..Problems::default()
        },
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

#[test]
fn outputs_of_a_device_that_cannot_play_share_a_warning_that_cinq_does_not_retry() {
    let fault = Fault::Unplayable("it takes no samples that cinq can play".to_owned());
    assert!(!fault.retried());
    assert_eq!(fault.state(), "Cannot open");
    assert_eq!(
        fault.reason(),
        Some("it takes no samples that cinq can play")
    );
    let app = app(
        &[dante(), dante()],
        vec![(OutputId(1), fault.clone()), (OutputId(2), fault.clone())],
    );
    let device = dante();
    assert_eq!(
        warnings(&app),
        [Warning::Device {
            device: &device,
            fault: &fault,
            outputs: vec![OutputId(1), OutputId(2)],
        }]
    );
    assert!(Fault::Refused(String::new()).retried());
}

#[test]
fn output_wholly_past_the_channels_of_its_device_counts_as_failing() {
    let app = app(
        &[dante(), dante(), dante()],
        vec![
            (OutputId(1), Fault::Channels(2)),
            (OutputId(2), Fault::Channels(2)),
            (OutputId(3), Fault::Channels(2)),
        ],
    );
    let app = [
        (OutputId(1), Channels::Mono(6)),
        (OutputId(2), Channels::Stereo(1, 2)),
        (OutputId(3), Channels::Stereo(2, 3)),
    ]
    .into_iter()
    .fold(app, |app, (id, channels)| App {
        workspace: app
            .workspace
            .apply(Command::EditOutput(id, OutputEdit::Channels(channels))),
        ..app
    });
    assert_eq!(failing(&app), [OutputId(1), OutputId(3)]);
}

#[test]
fn only_outputs_that_make_no_sound_count_as_failing() {
    let app = app(
        &[dante(), dante(), dante()],
        vec![
            (OutputId(1), missing()),
            (OutputId(2), Fault::Channels(16)),
            (OutputId(3), Fault::Unplayable("no samples".to_owned())),
        ],
    );
    assert_eq!(failing(&app), [OutputId(1), OutputId(3)]);
}

#[test]
fn one_channel_is_singular() {
    assert_eq!(Fault::Channels(1).state(), "Past its 1 channel");
    assert_eq!(Fault::Channels(2).state(), "Past its 2 channels");
}
