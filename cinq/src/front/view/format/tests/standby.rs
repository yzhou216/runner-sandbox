// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how the standby that is about to move is told

use super::*;

#[test]
fn standby_about_to_move_is_told_by_the_number_of_its_target_and_the_time_ahead() {
    [
        (2.94, "2.9s"),
        // A time ahead turns to a clock from a minute on
        (75.0, "1:15.0"),
        (59.97, "1:00.0"),
        // Billions of seconds are told in full, not cut at the limit of a counter
        (1e10, "166666666:40.0"),
    ]
    .into_iter()
    .for_each(|(seconds, told)| {
        assert_eq!(
            moving(&memo(5), seconds),
            format!("Standby moves to 5 in {told}"),
            "{seconds}"
        );
    });
}

#[test]
fn paused_cues_are_counted_for_the_line_of_the_chain_to_lead_with() {
    assert_eq!(paused(0), None);
    assert_eq!(paused(1).as_deref(), Some("1 paused"));
    assert_eq!(paused(4).as_deref(), Some("4 paused"));
}
