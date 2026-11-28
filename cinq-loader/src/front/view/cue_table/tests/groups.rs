// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the columns of a group and of the cues that it holds

use super::{
    super::{Column, columns::states::state},
    texts,
};
use crate::back::{Advance, Command, Cue, CueId, Edit, Kind, Workspace, fresh, loaded, sound};

/// Returns a list of a looping audio cue, a group that holds an audio cue of
/// ten seconds after a pre-wait of one, and a group that holds nothing
fn grouped(depth: u8) -> Workspace {
    let ten = |id: u32| {
        Cue::new(CueId(id), id.to_string(), Kind::Audio, "Rain")
            .file("rain.ogg")
            .pre_wait(1.0)
            .depth(depth)
            .advance(Advance::Continue)
            .post_wait(2.0)
            .build()
    };
    let group = |id: u32| Cue::new(CueId(id), id.to_string(), Kind::Group, "Group").build();
    let workspace = fresh(vec![group(1), ten(2), group(3)]);
    workspace.apply(loaded(2, "rain.ogg", sound(10.0)))
}

#[test]
fn a_group_tells_how_many_cues_it_holds_and_how_long_its_action_lasts() {
    let workspace = grouped(1);
    assert_eq!(
        texts(&workspace, Column::Target),
        ["1 cue", "rain.ogg", "0 cues"]
    );
    assert_eq!(texts(&workspace, Column::Action), ["0:11.0", "0:10.0", "-"]);
}

#[test]
fn the_action_of_a_group_holding_a_cue_that_never_ends_is_a_hold() {
    let workspace = grouped(1).apply(Command::Edit(CueId(2), Edit::Loop(true)));
    assert_eq!(texts(&workspace, Column::Action)[0], "hold");
}

#[test]
fn a_cue_in_a_group_shows_no_post_wait_and_no_then_setting() {
    let workspace = grouped(1);
    assert_eq!(texts(&workspace, Column::Post), ["", "", ""]);
    assert_eq!(texts(&workspace, Column::Then), ["", "", ""]);
    let outside = grouped(0);
    assert_eq!(texts(&outside, Column::Then)[1], "continue");
    assert_eq!(texts(&outside, Column::Post)[1], "0:02.0");
}

#[test]
fn a_start_cue_in_the_group_that_it_starts_reads_in_target() {
    let group = Cue::new(CueId(1), "1", Kind::Group, "Group").build();
    let start = Cue::new(CueId(2), "2", Kind::Start, "Start")
        .target(CueId(1))
        .depth(1)
        .build();
    let workspace = fresh(vec![group, start]);
    let list = &workspace.lists()[0];
    let (state, _) = state(&workspace, &list.cues[1], None, false);
    assert_eq!(state, "IN TARGET");
}

#[test]
fn a_collapsed_group_tells_its_count_in_the_target_column_and_leaves_its_name_bare() {
    let workspace = grouped(1).apply(Command::Collapse {
        cues: vec![CueId(1)],
        collapsed: true,
    });
    assert_eq!(texts(&workspace, Column::Name)[0], "Group");
    assert_eq!(texts(&workspace, Column::Target)[0], "1 cue");
}

#[test]
fn a_control_cue_that_names_itself_reads_bad_target_and_one_with_no_target_reads_none() {
    let stop = |id, target| {
        Cue::new(CueId(id), id.to_string(), Kind::Stop, "Stop")
            .target(CueId(target))
            .build()
    };
    let workspace = fresh(vec![stop(1, 1), stop(2, 9)]);
    let list = &workspace.lists()[0];
    let read = |at: usize| state(&workspace, &list.cues[at], None, false).0;
    assert_eq!(
        (read(0), read(1)),
        ("BAD TARGET".to_owned(), "NO TARGET".to_owned())
    );
}
