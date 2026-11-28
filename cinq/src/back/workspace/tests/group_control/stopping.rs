// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of stopping a group, the cue after it, and the cues that wait inside it

use super::*;

#[test]
fn stop_cue_on_a_group_stops_its_run_its_cues_and_the_cue_after_it() {
    let stopped = workspace([grouped(), vec![stop_cue(6, 1)]].concat())
        .apply(GO)
        .apply(Command::Tick(1.0))
        .apply(GO);
    assert!(running(&stopped).is_empty());
}

#[test]
fn stop_button_stops_a_group_and_the_cues_waiting_inside_it() {
    let started = workspace(grouped()).apply(GO);
    assert_eq!(running(&started), [1, 2, 3, 4, 5]);
    let stopped = started.apply(Command::Stop(CueId(1)));
    assert!(running(&stopped).is_empty());
}

#[test]
fn stopping_a_nested_group_leaves_the_group_around_it_to_its_other_cues() {
    let stopped = workspace(grouped())
        .apply(GO)
        .apply(Command::Tick(1.0))
        .apply(Command::Stop(CueId(3)));
    assert_eq!(running(&stopped), [1, 2, 5]);
}

#[test]
fn stopping_the_cue_that_waits_to_start_a_group_cancels_the_cues_of_the_group() {
    let waiting = workspace(vec![
        continuing_after(1, 3.0),
        group(2),
        within(1, audio(3)),
        within(1, group(4)),
        within(2, audio(5)),
    ])
    .apply(GO)
    .apply(Command::Tick(1.0));
    assert_eq!(running(&waiting), [1, 2, 3, 4, 5]);
    assert!(running(&waiting.apply(Command::Stop(CueId(1)))).is_empty());
}

#[test]
fn stop_cue_before_a_group_cancels_the_group_that_waits_for_it() {
    let waiting = workspace(vec![
        continuing_after(1, 3.0),
        group(2),
        within(1, audio(3)),
        stop_cue(4, 1),
    ])
    .apply(GO)
    .apply(Command::Tick(1.0))
    .apply(move_standby(4))
    .apply(GO);
    assert!(running(&waiting).is_empty());
}

#[test]
fn stopping_a_group_that_began_cancels_its_waiting_cues_and_the_cue_after_it() {
    let late = pre_waiting(3.0, within(1, audio(3)));
    let begun = workspace(vec![
        Cue {
            advance: Advance::Continue,
            post_wait: 4.0.into(),
            ..group(1)
        },
        within(1, audio(2)),
        late,
        audio(4),
    ])
    .apply(GO)
    .apply(Command::Tick(1.0));
    assert!(running(&begun.apply(Command::Stop(CueId(1)))).is_empty());
}

#[test]
fn end_of_a_group_stops_nothing() {
    let short = Cue {
        end: 1.0.into(),
        ..within(1, audio(2))
    };
    let ended = workspace(vec![
        Cue {
            advance: Advance::Continue,
            post_wait: 5.0.into(),
            ..group(1)
        },
        short,
        audio(3),
    ])
    .apply(GO)
    .apply(Command::Tick(2.0));
    assert_eq!(running(&ended), [3]);
    assert_phase(&ended, 3, Phase::Scheduled { starts_in: 3.0 });
}
