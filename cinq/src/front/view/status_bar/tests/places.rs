// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how the hints of back and forward name the place they go to

use unicode_width::UnicodeWidthStr;

use super::{super::hints::place, opened};
use crate::{
    back::{Cue, CueId, CueList, Kind, OutputId, Workspace},
    front::{App, keymap::Preset, update::Spot},
};

fn on_show(outputs_shown: bool) -> App {
    let memo = |id: u32| Cue::new(CueId(id), id.to_string(), Kind::Memo, "Note").build();
    App {
        workspace: Workspace::new(vec![
            CueList::new("Main", vec![memo(1), memo(2)]),
            CueList::new("A cue list with a very long name", vec![memo(3)]),
        ]),
        outputs_shown,
        ..opened(Preset::Vi, false, false)
    }
}

#[test]
fn the_outputs_are_named_by_what_they_are_and_never_by_an_output() {
    let app = on_show(false);
    assert_eq!(place(&app, Spot::Outputs), "the outputs");
    assert_eq!(place(&app, Spot::Output(OutputId(1))), "the outputs");
}

#[test]
fn a_cue_of_the_list_on_show_is_named_by_its_number_and_any_other_by_its_list() {
    let app = on_show(false);
    assert_eq!(place(&app, Spot::Cue(CueId(2))), "cue 2");
    assert_eq!(place(&app, Spot::Cue(CueId(3))), "A cue list with…");
    assert_eq!(place(&app, Spot::List(0)), "Main");
    // The outputs hide the list, so that its cues are named by it
    assert_eq!(place(&on_show(true), Spot::Cue(CueId(2))), "Main");
}

#[test]
fn a_long_name_of_a_list_is_cut_to_keep_the_hint_short() {
    let app = on_show(false);
    assert!(place(&app, Spot::List(1)).width() <= 16);
}
