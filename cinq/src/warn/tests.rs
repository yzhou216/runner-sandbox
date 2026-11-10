// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of when a warning comes out

use super::Warnings;

#[test]
fn warning_comes_out_once_after_cinq() {
    let (warnings, first) = Warnings::default().warned("a.mp4: skipped its tags");
    assert_eq!(first, "cinq: a.mp4: skipped its tags\n");
    let (warnings, again) = warnings.warned("a.mp4: skipped its tags");
    assert_eq!(again, "");
    let (_, other) = warnings.warned("b.mp4: skipped its tags");
    assert_eq!(other, "cinq: b.mp4: skipped its tags\n");
}

#[test]
fn warnings_wait_for_the_main_screen_in_the_order_they_came() {
    let (warnings, _) = Warnings::default().holding();
    let (warnings, first) = warnings.warned("a.mp4: skipped its tags");
    let (warnings, second) = warnings.warned("b.mp4: skipped its tags");
    let (warnings, again) = warnings.warned("a.mp4: skipped its tags");
    assert_eq!(
        (first, second, again),
        (String::new(), String::new(), String::new())
    );
    let (warnings, waited) = warnings.released();
    assert_eq!(
        waited,
        "cinq: a.mp4: skipped its tags\ncinq: b.mp4: skipped its tags\n"
    );
    let (_, after) = warnings.warned("c.mp4: skipped its tags");
    assert_eq!(after, "cinq: c.mp4: skipped its tags\n");
}

#[test]
fn warning_that_waited_is_not_written_again_on_the_main_screen() {
    let (warnings, _) = Warnings::default().holding();
    let (warnings, _) = warnings.warned("a.mp4: skipped its tags");
    let (warnings, _) = warnings.released();
    let (_, again) = warnings.warned("a.mp4: skipped its tags");
    assert_eq!(again, "");
}

#[test]
fn release_without_waiting_warnings_writes_nothing() {
    assert_eq!(Warnings::default().released().1, "");
    assert_eq!(Warnings::default().holding().0.released().1, "");
}
