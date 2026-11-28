// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the commands that OSC gives for single cues in show mode, and of
//! finding a cue by its number

use super::*;
use crate::back::Change;

fn numbered(workspace: &Workspace, number: &str) -> Option<u32> {
    workspace.numbered(number).map(|cue| cue.id.0)
}

#[test]
fn cue_goes_by_its_number_with_surrounding_spaces_ignored() {
    let workspace = workspace(vec![audio(1), audio(2)])
        .apply(edit(1, Edit::Number(" 1.5 ".to_owned())))
        .apply(edit(2, Edit::Number("A".to_owned())));
    assert_eq!(numbered(&workspace, "1.5"), Some(1));
    assert_eq!(numbered(&workspace, "  1.5"), Some(1));
    assert_eq!(numbered(&workspace, "A"), Some(2));
    assert_eq!(numbered(&workspace, "a"), None);
    assert_eq!(numbered(&workspace, "1.50"), None);
    assert_eq!(numbered(&workspace, "7"), None);
}

#[test]
fn first_cue_with_a_number_is_meant_in_the_order_of_lists_and_rows() {
    let workspace = lists_of(vec![vec![audio(1), audio(2)], vec![audio(3), audio(4)]])
        .apply(edit(2, Edit::Number("9".to_owned())))
        .apply(edit(3, Edit::Number("9".to_owned())))
        .apply(edit(4, Edit::Number("9".to_owned())));
    assert_eq!(numbered(&workspace, "9"), Some(2));
}

#[test]
fn no_cue_is_the_one_with_no_number() {
    let workspace = workspace(vec![audio(1)]).apply(edit(1, Edit::Number(String::new())));
    assert_eq!(numbered(&workspace, ""), None);
    assert_eq!(numbered(&workspace, "  "), None);
}

#[test]
fn start_hold_and_go_to_act_in_show_mode() {
    let started = workspace(vec![audio(1), audio(2)])
        .apply(rename(2, "Last"))
        .apply(Command::Start(CueId(1)));
    // Starting leaves standby and the undo history alone
    assert_eq!(standby(&started), Some(1));
    assert!(matches!(started.undoable(), Some(Change::Edited { .. })));
    let paused = started
        .apply(Command::ShowMode(true))
        .apply(Command::Hold(CueId(1)));
    assert!(paused.active_of(CueId(1)).unwrap().paused);
    let resumed = paused.apply(Command::Start(CueId(1)));
    assert_eq!(running(&resumed), [1]);
    assert!(!resumed.active_of(CueId(1)).unwrap().paused);
    let moved = resumed.apply(Command::GoTo(CueId(2)));
    assert_eq!(standby(&moved), Some(2));
    let lists = lists_of(vec![vec![audio(1), audio(2)], vec![audio(3), audio(4)]])
        .apply(Command::ShowMode(true))
        .apply(Command::GoTo(CueId(4)));
    // Another list keeps its standby
    assert_eq!(
        (standby_of(&lists, 0), standby_of(&lists, 1)),
        (Some(1), Some(4))
    );
}

#[test]
fn resume_fires_nothing_where_a_cue_is_not_running() {
    let workspace = workspace(vec![audio(1)]).apply(Command::Resume(CueId(1)));
    assert!(workspace.active().is_empty());
    let held = workspace.apply(Command::Hold(CueId(1)));
    assert!(held.active().is_empty());
}

#[test]
fn start_fires_a_cue_with_its_chain() {
    let started = workspace(vec![audio(1), continuing_after(2, 2.0), audio(3), audio(4)])
        .apply(Command::Start(CueId(2)));
    assert_eq!(running(&started), [2, 3]);
    assert_eq!(standby(&started), Some(1));
    assert_phase(&started, 3, Phase::Scheduled { starts_in: 2.0 });
}

#[test]
fn start_resumes_a_paused_cue_where_it_was() {
    let paused = workspace(vec![audio(1), audio(2)])
        .apply(GO)
        .apply(Command::ShowMode(true))
        .apply(Command::Tick(1.0))
        .apply(Command::Hold(CueId(1)));
    let resumed = paused.apply(Command::Start(CueId(1)));
    assert_phase(&resumed, 1, Phase::Running { remaining: 9.0 });
}

#[test]
fn hold_and_resume_pause_and_free_a_run_with_the_cues_it_would_start() {
    let held = workspace(vec![continuing_after(1, 6.0), audio(2)])
        .apply(GO)
        .apply(Command::Tick(1.0))
        .apply(Command::Hold(CueId(1)))
        .apply(Command::Tick(3.0));
    assert_phase(&held, 1, Phase::Running { remaining: 9.0 });
    assert_phase(&held, 2, Phase::Scheduled { starts_in: 5.0 });
    let resumed = held
        .apply(Command::Resume(CueId(1)))
        .apply(Command::Tick(1.0));
    assert_phase(&resumed, 1, Phase::Running { remaining: 8.0 });
    assert_phase(&resumed, 2, Phase::Scheduled { starts_in: 4.0 });
}
