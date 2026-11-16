// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of which outputs that cannot play the Levels tab counts, and of how
//! it names them

use super::{named, silent};
use crate::{
    back::{Command, Cue, CueId, CueList, Fault, Kind, OutputId, Sends, Workspace},
    front::App,
};

#[test]
fn only_outputs_that_the_cue_sends_to_count_as_unable_to_play() {
    let sends: Sends = [(OutputId(1), 0.0)].into_iter().collect();
    let cue = Cue {
        sends: Some(sends.clone()),
        ..Cue::new(CueId(1), "1", Kind::Audio, "Music").build()
    };
    let workspace =
        Workspace::new(vec![CueList::new("Main", vec![cue])]).apply(Command::AddOutput {
            at: 1,
            count: 1,
            anchor: None,
        });
    let missing = || Fault::Missing("there is no audio device".to_owned());
    let app = App {
        workspace,
        audio_problems: vec![(OutputId(1), missing()), (OutputId(2), missing())],
        ..App::default()
    };
    assert_eq!(silent(&app, &sends), [&missing()]);
}

#[test]
fn an_output_that_cannot_play_is_named_with_what_is_wrong_with_it() {
    assert_eq!(named("House L", None), "House L");
    let faults = [
        (Fault::Missing("gone".to_owned()), "House L (not found)"),
        (Fault::Refused("busy".to_owned()), "House L (cannot open)"),
        (Fault::Unplayable("no".to_owned()), "House L (cannot open)"),
        (Fault::Stopped("pulled".to_owned()), "House L (stopped)"),
        (Fault::Unanswered, "House L (not answering)"),
        (Fault::Channels(2), "House L (past its channels)"),
    ];
    faults
        .iter()
        .for_each(|(fault, words)| assert_eq!(named("House L", Some(fault)), *words));
}
