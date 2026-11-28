// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of a search matching the text of every column that the list shows
//! of a row, as Vim matches the whole line, but for the state, in the cue
//! list and in the outputs

use super::{after, one_list, output_editing::on, output_rows::at, search::searched};
use crate::{
    back::{
        Advance, Channels, Command, Cue, CueId, Device, Kind, OutputEdit, OutputId, loaded, sound,
    },
    front::{App, Message},
};

/// Returns the app with the cue list on screen, the first of five cues
/// selected, and a search line open: a sound with a file, a fade of it that
/// continues, a pause that waits before it starts, a memo, and a stop
fn listed() -> App {
    let thunder = Cue::new(CueId(1), "A1", Kind::Audio, "Thunder")
        .file("storm/rumble.ogg")
        .build();
    let fade = Cue {
        advance: Advance::Continue,
        ..Cue::new(CueId(2), "A2", Kind::Fade, "Out")
            .target(CueId(1))
            .build()
    };
    let waiting = Cue {
        pre_wait: 3.0.into(),
        post_wait: 4.5.into(),
        ..Cue::new(CueId(3), "A3", Kind::Memo, "Note").build()
    };
    let stop = Cue::new(CueId(4), "A4", Kind::Stop, "End")
        .target(CueId(1))
        .build();
    let app = one_list(vec![thunder, fade, waiting, stop], Some(CueId(4)));
    App {
        workspace: app
            .workspace
            .apply(loaded(1, "storm/rumble.ogg", sound(12.0))),
        ..app
    }
    .handled(Message::Search { backward: false })
    .0
}

fn found(text: &str) -> Option<u32> {
    searched(listed(), text).selected.map(|id| id.0)
}

#[test]
fn a_search_matches_the_number_type_name_target_action_and_then_of_a_cue() {
    // Each text lies in one column of one cue, and the search wraps to it
    assert_eq!(found("a2"), Some(2), "number");
    assert_eq!(found("audio"), Some(1), "type");
    assert_eq!(found("thunder"), Some(1), "name");
    assert_eq!(found("rumble"), Some(1), "target of a file");
    assert_eq!(found("ogg"), Some(1), "target of a file");
    assert_eq!(found("0:12"), Some(1), "action");
    assert_eq!(found("continue"), Some(2), "what follows");
    assert_eq!(found("fade"), Some(2), "type");
}

#[test]
fn a_search_leaves_out_the_waits_and_the_state() {
    // Selection stays on the last cue, which a miss leaves alone
    assert_eq!(found("0:03"), Some(4), "pre-wait");
    assert_eq!(found("0:04"), Some(4), "post-wait");
    // Cue 1 is the standby of its list, which the state column tells
    assert_eq!(found("next"), Some(4), "state");
}

/// Returns the app with the outputs on screen, the first selected, the
/// third on a device that this test names, the second with a level for new
/// cues, and a search line open
fn outputs() -> App {
    let app = on(1);
    let sub = app.workspace.outputs()[2].id;
    let device = Device::Named {
        id: "alsa:hw:USB".to_owned(),
        name: "Studio USB".to_owned(),
    };
    let workspace = app
        .workspace
        .apply(Command::EditOutput(sub, OutputEdit::Device(device)))
        .apply(Command::EditOutput(
            OutputId(2),
            OutputEdit::NewCues(Some(-6.0)),
        ));
    after(
        App { workspace, ..app },
        [Message::Search { backward: false }],
    )
}

fn row(text: &str) -> usize {
    at(&searched(outputs(), text))
}

#[test]
fn a_search_matches_the_name_device_channels_and_new_cues_of_an_output() {
    assert_eq!(row("monitor"), 4, "name");
    assert_eq!(row("studio"), 3, "device");
    assert_eq!(row("usb"), 3, "device");
    assert_eq!(row("-6.0"), 2, "new cues");
    // Channels read from one, as the table shows them
    let mono = outputs().workspace.apply(Command::EditOutput(
        OutputId(5),
        OutputEdit::Channels(Channels::Mono(6)),
    ));
    let app = searched(
        App {
            workspace: mono,
            ..outputs()
        },
        "7",
    );
    assert_eq!(at(&app), 5, "channels");
    // Number of the row is no text of the output, nor is its state
    assert_eq!(row("5"), 1, "no match leaves the selection");
}
