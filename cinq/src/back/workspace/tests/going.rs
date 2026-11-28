// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of `GoTo` cues, which move standby to their target, and of the three
//! control cues in show mode and in show files

use super::*;

#[test]
fn goto_cue_moves_standby_in_its_own_list_and_fires_nothing() {
    let workspace = workspace(vec![audio(1), audio(2), goto_cue(3, 1)])
        .apply(move_standby(3))
        .apply(GO);
    assert_eq!(standby(&workspace), Some(1));
    assert!(workspace.active().is_empty());
}

#[test]
fn goto_cue_moves_the_standby_of_the_list_that_holds_its_target() {
    let workspace = lists_of(vec![vec![goto_cue(1, 3)], vec![audio(2), audio(3)]]).apply(GO);
    assert_eq!(standby_of(&workspace, 1), Some(3));
    assert_eq!(standby_of(&workspace, 0), None);
}

#[test]
fn goto_cue_in_a_chain_sets_the_standby_that_go_leaves() {
    let chain = workspace(vec![continuing(1), goto_cue(2, 4), audio(3), audio(4)]);
    assert_eq!(standby(&chain.apply(GO)), Some(4));
    let delayed = workspace(vec![
        continuing_after(1, 2.0),
        goto_cue(2, 4),
        audio(3),
        audio(4),
    ])
    .apply(GO);
    // GO has moved standby past the chain, and the GoTo cue acts later
    assert_eq!(standby(&delayed), Some(3));
    assert_eq!(standby(&delayed.apply(Command::Tick(2.0))), Some(4));
}

#[test]
fn goto_move_cannot_be_undone_and_drops_earlier_moves_of_that_list() {
    let moved = lists_of(vec![
        vec![audio(1), audio(2), audio(3)],
        vec![goto_cue(4, 3)],
    ])
    .apply(move_standby(2))
    .apply(Command::Go { list: 1 });
    assert_eq!(standby_of(&moved, 0), Some(3));
    assert!(moved.undoable().is_none());
    assert_eq!(standby_of(&moved.apply(Command::Undo), 0), Some(3));
}

#[test]
fn control_cues_work_in_show_mode() {
    let show = workspace(vec![
        audio(1),
        pause_cue(2, 1),
        start_cue(3, 1),
        goto_cue(4, 1),
    ])
    .apply(Command::ShowMode(true))
    .apply(GO);
    let paused = |workspace: &Workspace| workspace.active_of(CueId(1)).map(|active| active.paused);
    assert_eq!(paused(&show), Some(false));
    let held = show.apply(GO);
    assert_eq!(paused(&held), Some(true));
    let resumed = held.apply(GO);
    assert_eq!(paused(&resumed), Some(false));
    assert_eq!(standby(&resumed.clone().apply(GO)), Some(1));
    assert_eq!(running(&resumed), [1]);
}

#[test]
fn show_file_names_start_pause_and_goto_cues_by_their_kind() {
    let show = workspace(vec![
        audio(1),
        start_cue(2, 1),
        pause_cue(3, 1),
        goto_cue(4, 1),
    ])
    .directory("/show");
    let text = shown(&show);
    ["start", "pause", "goto"].into_iter().for_each(|kind| {
        assert!(text.contains(&format!("kind = \"{kind}\"")), "{text}");
    });
}

#[test]
fn goto_cue_that_has_yet_to_act_tells_where_it_moves_standby_and_when() {
    let jump = pre_waiting(4.0, goto_cue(1, 3));
    let waiting = lists_of(vec![vec![jump, audio(2)], vec![audio(3), audio(4)]]).apply(GO);
    let moving = |workspace: &Workspace, list| {
        workspace
            .moving(list)
            .map(|(target, seconds)| (target.id.0, seconds))
    };
    assert_eq!(moving(&waiting, 1), Some((3, 4.0)));
    assert_eq!(moving(&waiting, 0), None);
    let later = waiting.apply(Command::Tick(1.5));
    assert_eq!(moving(&later, 1), Some((3, 2.5)));
    assert_eq!(moving(&later.clone().apply(Command::Pause(true)), 1), None);
    assert_eq!(
        moving(&later.clone().apply(Command::Stop(CueId(1))), 1),
        None
    );
    assert_eq!(moving(&later.apply(Command::Tick(3.0)), 1), None);
}
