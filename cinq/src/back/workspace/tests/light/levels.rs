// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the levels that light cues fade to and of the DMX they make

use super::*;
use crate::back::PatchEdit;

#[test]
fn light_cue_fades_its_instruments_and_leaves_them_where_it_ends() {
    let fading = lit(vec![light(1, &[(1, 80.0)])]).apply(GO).apply(tick(1.0));
    assert_eq!(level(&fading, 1), 20.0);
    let done = fading.apply(tick(3.0));
    assert!(done.active().is_empty());
    assert_eq!(level(&done, 1), 80.0);
    assert_eq!(level(&done, 2), 0.0);
    assert_eq!(done.universes()[&1][..3], [204, 0, 0]);
}

#[test]
fn newer_light_cue_takes_over_only_the_instruments_it_names() {
    let ticks = |workspace: Workspace, count: usize| {
        (0..count).fold(workspace, |workspace, _| workspace.apply(tick(0.025)))
    };
    let near = |actual: f32, expected: f32| (actual - expected).abs() < 0.01;
    let first = ticks(
        lit(vec![
            light(1, &[(1, 100.0), (2, 100.0)]),
            light(2, &[(1, 0.0)]),
        ])
        .apply(GO),
        40,
    );
    assert!(near(level(&first, 1), 25.0));
    // Instrument 1 fades on from 25 under the second cue alone, and
    // instrument 2 stays with the first
    let taken = ticks(first.apply(GO), 80);
    assert!(near(level(&taken, 1), 12.5));
    assert!(near(level(&taken, 2), 75.0));
    let done = ticks(taken, 200);
    assert_eq!(level(&done, 1), 0.0);
    assert_eq!(level(&done, 2), 100.0);
}

#[test]
fn light_cue_of_no_duration_sets_its_instruments_at_once() {
    let cue = Cue::new(CueId(1), "1", Kind::Light, "Light")
        .duration(0.0)
        .lights([(InstrumentId(1), 80.0)].into())
        .build();
    let workspace = lit(vec![cue]).apply(GO).apply(tick(0.025));
    assert!(workspace.active().is_empty());
    assert_eq!(level(&workspace, 1), 80.0);
    assert_eq!(level(&workspace, 2), 0.0);
    assert_eq!(workspace.universes()[&1][..2], [204, 0]);
}

#[test]
fn paused_or_stopped_light_cue_leaves_its_instruments_where_they_are() {
    let fading = lit(vec![light(1, &[(1, 80.0)])]).apply(GO).apply(tick(2.0));
    let paused = fading.clone().apply(Command::Pause(true)).apply(tick(1.0));
    assert_eq!(level(&paused, 1), 40.0);
    let resumed = paused.apply(Command::Pause(false)).apply(tick(1.0));
    assert_eq!(level(&resumed, 1), 60.0);
    let panicked = fading.apply(Command::Panic).apply(tick(1.0));
    assert!(panicked.active().is_empty());
    assert_eq!(level(&panicked, 1), 40.0);
}

#[test]
fn instruments_on_one_address_set_it_to_the_highest_of_their_levels() {
    let shared = Command::Patch(PatchEdit::Address(InstrumentId(2), 1, 1));
    let workspace = lit(vec![light(1, &[(1, 20.0), (2, 100.0)])])
        .apply(shared)
        .apply(GO)
        .apply(tick(4.0));
    assert_eq!(workspace.universes()[&1][..2], [255, 0]);
}
