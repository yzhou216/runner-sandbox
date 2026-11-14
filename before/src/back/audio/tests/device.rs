// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of opening audio devices by their IDs, which need no audio
//! device, since no host of cpal is named cinq

use crate::back::{Audio, Channels, Command, Device, OutputEdit, OutputId, READY_AHEAD, Workspace};

const MAIN: OutputId = OutputId(1);

fn problems(audio: &mut Audio, workspace: Workspace) -> Vec<(OutputId, String)> {
    audio.sync(workspace, READY_AHEAD);
    audio.problems()
}

#[test]
fn output_on_a_device_that_is_not_there_tells_why_nothing_is_heard() {
    let mut audio = Audio::open(Some("cinq:none"));
    assert_eq!(
        problems(&mut audio, Workspace::default()),
        [(MAIN, "there is no audio device cinq:none".to_owned())]
    );
}

#[test]
fn routing_to_another_device_opens_it() {
    let mut audio = Audio::open(Some("cinq:first"));
    problems(&mut audio, Workspace::default());
    audio.route(Some("cinq:second"));
    assert_eq!(
        problems(&mut audio, Workspace::default()),
        [(MAIN, "there is no audio device cinq:second".to_owned())]
    );
}

#[test]
fn output_on_a_named_device_goes_by_its_name_where_the_id_finds_none() {
    let named = Device::Named {
        id: "cinq:desk".to_owned(),
        name: "Desk".to_owned(),
    };
    let workspace = Workspace::default()
        .apply(Command::EditOutput(MAIN, OutputEdit::Device(named)))
        .apply(Command::EditOutput(
            MAIN,
            OutputEdit::Channels(Channels::Stereo(4, 5)),
        ));
    let mut audio = Audio::open(None);
    assert_eq!(
        problems(&mut audio, workspace),
        [(MAIN, "there is no audio device Desk".to_owned())]
    );
}
