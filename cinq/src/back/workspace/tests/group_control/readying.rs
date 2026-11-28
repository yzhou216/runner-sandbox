// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of readying the cues of a group ahead

use super::*;

#[test]
fn start_cue_on_a_group_readies_the_cues_it_holds_ahead() {
    let workspace = workspace(vec![
        pre_waiting(1.0, start_cue(1, 2)),
        pre_waiting(2.0, group(2)),
        within(1, audio(3)),
        within(1, group(4)),
        within(2, audio(5)),
    ]);
    assert_eq!(
        ahead(&workspace),
        [(1, 0.0), (2, 1.0), (3, 3.0), (4, 3.0), (5, 3.0)]
    );
}

#[test]
fn cues_of_the_next_group_are_readied_ahead_with_their_delays() {
    let workspace = workspace(vec![
        pre_waiting(2.0, group(1)),
        within(1, audio(2)),
        within(1, audio(3)),
    ]);
    assert_eq!(ahead(&workspace), [(1, 0.0), (2, 2.0), (3, 2.0)]);
}
