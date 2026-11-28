// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of which run starts which: the cues of a group, the cue after it, GO
//! with standby, and a start cue

use super::*;

/// Returns the run that starts the run of cue `id`
fn started_by(workspace: &Workspace, id: u32) -> Option<u32> {
    workspace
        .active_of(CueId(id))
        .and_then(|active| active.after)
}

#[test]
fn run_of_a_group_starts_the_cues_it_holds_and_the_cue_that_follows_it() {
    let fired = workspace(vec![
        then(1, Advance::Continue, 0.0, 4.0),
        within(1, audio(2)),
        within(1, group(3)),
        within(2, audio(4)),
        continuing_after(5, 2.0),
        audio(6),
    ])
    .apply(GO);
    assert_eq!(running(&fired), [1, 2, 3, 4, 5, 6]);
    let after = |id| started_by(&fired, id);
    let run = |id| Some(run_of(&fired, id));
    assert_eq!(after(1), None);
    assert_eq!(after(2), run(1));
    assert_eq!(after(3), run(1));
    assert_eq!(after(4), run(3));
    assert_eq!(after(5), run(1));
    assert_eq!(after(6), run(5));
}

#[test]
fn chain_that_reaches_a_group_starts_it_by_the_cue_before_it() {
    let fired = workspace(vec![continuing(1), group(2), within(1, audio(3)), audio(4)]).apply(GO);
    assert_eq!(started_by(&fired, 2), Some(run_of(&fired, 1)));
    assert_eq!(started_by(&fired, 3), Some(run_of(&fired, 2)));
}

#[test]
fn go_with_standby_in_a_group_fires_that_cue_alone_and_moves_to_the_next_row() {
    let fired = workspace(vec![
        group(1),
        within(1, continuing(2)),
        within(1, group(3)),
        within(2, audio(4)),
        within(1, audio(5)),
        audio(6),
    ])
    .apply(move_standby(2))
    .apply(GO);
    assert_eq!(running(&fired), [2]);
    assert_eq!(standby(&fired), Some(3));
    // Group inside it fires with its own cues, and standby leaves it
    let inner = fired.apply(GO);
    assert_eq!(running(&inner), [2, 3, 4]);
    assert_eq!(started_by(&inner, 3), None);
    assert_eq!(started_by(&inner, 4), Some(run_of(&inner, 3)));
    assert_eq!(standby(&inner), Some(5));
}

#[test]
fn group_in_another_list_runs_when_a_start_cue_fires_it() {
    let fired = lists_of(vec![
        vec![start_cue(1, 2)],
        vec![group(2), within(1, audio(3))],
    ])
    .apply(GO)
    .apply(Command::Tick(1.0));
    assert_eq!(running(&fired), [2, 3]);
    assert_eq!(started_by(&fired, 3), Some(run_of(&fired, 2)));
    assert_eq!(standby_of(&fired, 1), Some(2));
}
