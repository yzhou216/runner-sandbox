// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the cues that lie in the group they act on

use super::*;

#[test]
fn start_cue_in_the_group_that_it_starts_cannot_play() {
    let list = |depth| {
        workspace(vec![
            group(1),
            within(1, group(2)),
            within(depth, start_cue(3, 1)),
            within(2, audio(4)),
        ])
    };
    // At the depth of the group's cues, and deeper
    for depth in [1, 2] {
        let list = list(depth);
        assert_eq!(problems(&list), [(3, Problem::InTarget)]);
        assert_eq!(
            Problem::InTarget.to_string(),
            "lies in the group that it starts"
        );
        let fired = list.apply(GO);
        assert!(!running(&fired).contains(&3), "GO skips the start cue");
    }
    let outside = workspace(vec![group(1), within(1, audio(2)), start_cue(3, 1)]);
    assert_eq!(problems(&outside), []);
}

#[test]
fn stop_pause_and_go_to_cues_may_lie_in_their_own_group() {
    let workspace = workspace(vec![
        group(1),
        within(1, stop_cue(2, 1)),
        within(1, pause_cue(3, 1)),
        within(1, goto_cue(4, 1)),
    ]);
    assert_eq!(workspace.problems().count(), 0);
}
