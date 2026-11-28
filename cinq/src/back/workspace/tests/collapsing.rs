// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of collapsed groups, which hide the cues that they hold in the cue
//! list

use super::*;

/// Returns a group, two levels of cues in it, and a cue below it
fn nested() -> Workspace {
    workspace(vec![
        group(1),
        within(1, audio(2)),
        within(1, group(3)),
        within(2, audio(4)),
        audio(5),
    ])
}

#[test]
fn collapsed_group_stands_for_the_cues_that_it_holds() {
    let workspace = nested().apply(collapse(&[3], true));
    let groups = workspace.lists()[0].groups();
    assert_eq!(groups.shown_as(), [0, 1, 2, 2, 4]);
    assert_eq!(groups.shown(), [0, 1, 2, 4]);
    let outer = workspace.apply(collapse(&[1], true));
    let groups = outer.lists()[0].groups();
    assert_eq!(groups.shown_as(), [0, 0, 0, 0, 4]);
    assert_eq!(groups.shown(), [0, 4]);
    assert_eq!(groups.hiding(CueId(4)), [CueId(3), CueId(1)]);
    assert!(groups.hiding(CueId(5)).is_empty());
}

#[test]
fn collapsing_is_no_change_to_undo_and_leaves_other_cues_alone() {
    let workspace = nested().apply(collapse(&[1, 2], true));
    assert!(workspace.undoable().is_none());
    let collapsed: Vec<bool> = workspace.lists()[0]
        .cues
        .iter()
        .map(|cue| cue.collapsed)
        .collect();
    assert_eq!(collapsed, [true, false, false, false, false]);
    let shown = workspace
        .apply(Command::ShowMode(true))
        .apply(collapse(&[1], false));
    assert!(!shown.lists()[0].cues[0].collapsed);
}

#[test]
fn show_file_writes_collapsed_only_for_collapsed_groups() {
    let workspace = nested().apply(collapse(&[3], true));
    let text = shown(&workspace);
    assert_eq!(text.matches("collapsed = true").count(), 1);
    assert!(!shown(&nested()).contains("collapsed"));
}
