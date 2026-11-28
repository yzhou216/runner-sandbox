// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the marks of the groups in the cue list

use super::{flip, mark};
use crate::{
    back::{Cue, CueId, Kind},
    front::Message,
};

fn group(collapsed: bool) -> Cue {
    Cue {
        collapsed,
        ..Cue::new(CueId(7), "7", Kind::Group, "Storm").build()
    }
}

#[test]
fn an_expanded_group_reads_a_minus_and_a_collapsed_one_a_plus() {
    assert_eq!(mark(&group(false)), "- ");
    assert_eq!(mark(&group(true)), "+ ");
}

#[test]
fn a_click_on_the_mark_of_a_group_flips_that_group() {
    assert!(matches!(
        flip(&group(true)),
        Some(Message::FlipGroup(CueId(7)))
    ));
    assert!(matches!(
        flip(&group(false)),
        Some(Message::FlipGroup(CueId(7)))
    ));
}

#[test]
fn a_cue_that_is_no_group_has_no_mark_to_click() {
    let memo = Cue::new(CueId(8), "8", Kind::Memo, "Note").build();
    assert!(flip(&memo).is_none());
}
