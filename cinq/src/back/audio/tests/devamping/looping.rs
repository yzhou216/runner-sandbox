// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of a loop section going round its loop, and of a loop that ends with
//! its clip

use super::*;

#[test]
fn a_loop_section_goes_round_its_loop_and_never_back_before_its_start() {
    let (directory, workspace) = vamp(RATE, 2.5, (0.5, 1.5), 0.0);
    let (_, seen) = Rig::start(&directory, workspace).run(240, |_, workspace| workspace);
    assert_eq!(wraps(&seen), 5, "one at the end of each pass but the lead");
    let inside = seen.iter().skip_while(|seen| seen.position < 0.5);
    assert!(inside.clone().all(|seen| seen.position >= 0.5 - 1e-6));
    assert!(seen.iter().all(|seen| seen.position < 1.5 + 1e-6));
    assert!(seen.iter().all(|seen| seen.heard), "the sound never ends");
    assert!(seen.iter().all(|seen| !seen.cut));
    let off = drift(&seen);
    assert!(
        off < 0.06,
        "the clock follows the sound across passes: {off}"
    );
}

#[test]
fn a_loop_that_ends_with_its_clip_ends_the_run_at_the_pass_end() {
    let (directory, workspace) = vamp(RATE, 2.5, (0.5, 2.5), 0.0);
    let (rig, seen) = Rig::start(&directory, workspace).run(140, go_at(20));
    assert_eq!(wraps(&seen), 0, "the first pass is the last");
    assert!(seen.iter().all(|seen| seen.position <= 2.5 + 1e-6));
    assert!(
        seen.iter().all(|seen| !seen.cut),
        "a devamp is no early stop"
    );
    assert!(rig.workspace.active_of(CueId(1)).is_none(), "the run ends");
    assert!(!seen.last().expect("the test steps").heard);
}
