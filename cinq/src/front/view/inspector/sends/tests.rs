// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of which outputs that cannot play the Levels tab counts

use super::silent;
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
    let workspace = Workspace::new(vec![CueList::new("Main", vec![cue])])
        .apply(Command::AddOutput { at: 1, count: 1 });
    let missing = || Fault::Missing("there is no audio device".to_owned());
    let app = App {
        workspace,
        audio_problems: vec![(OutputId(1), missing()), (OutputId(2), missing())],
        ..App::default()
    };
    assert_eq!(silent(&app, &sends), [&missing()]);
}
