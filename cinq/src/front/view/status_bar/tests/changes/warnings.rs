// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the warnings that a change to the outputs raises

use super::*;
use crate::front::Problems;

#[test]
fn a_cue_that_sends_to_no_output_left_is_a_warning() {
    let sends: Sends = [(OutputId(2), 0.0)].into();
    let show = rain().apply(Command::Edit(CueId(1), Edit::Sends(Some(sends))));
    let warnings = |workspace: Workspace| {
        crate::front::view::warnings(&App {
            workspace,
            ..App::default()
        })
    };
    let counted = warnings(show.clone());
    let stranded = show.apply(Command::DeleteOutputs(vec![OutputId(2)]));
    assert_eq!(warnings(stranded.clone()), counted + 1);
    assert_eq!(warnings(stranded.apply(Command::Undo)), counted);
}

#[test]
fn light_output_that_cannot_go_out_counts_as_a_warning_until_a_round_gets_out() {
    let count = |light| {
        crate::front::view::warnings(&App {
            problems: Problems {
                light,
                ..Problems::default()
            },
            ..App::default()
        })
    };
    assert_eq!(
        count(Some("no network interface".to_owned())),
        count(None) + 1
    );
}
