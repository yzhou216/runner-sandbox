// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how long a group runs and what it shows: the cues it holds, a loop,
//! and the groups inside it

use super::*;

#[test]
fn group_with_its_cues_runs_while_any_of_them_does() {
    let late = pre_waiting(2.0, within(1, audio(3)));
    let fired = workspace(vec![
        then(1, Advance::Manual, 1.0, 0.0),
        within(1, audio(2)),
        late,
    ])
    .apply(GO);
    assert_phase(&fired, 1, Phase::PreWait { remaining: 1.0 });
    assert_phase(&fired, 2, Phase::Scheduled { starts_in: 1.0 });
    let begun = fired.apply(Command::Tick(1.0));
    assert_phase(&begun, 1, Phase::Running { remaining: 12.0 });
    let half = begun.apply(Command::Tick(5.0));
    assert_phase(&half, 1, Phase::Running { remaining: 7.0 });
    assert_eq!(half.active_of(CueId(1)).unwrap().progress(), 5.0 / 12.0);
    // First cue has ended, and the group lasts for the second
    let first = half.apply(Command::Tick(5.0));
    assert_eq!(running(&first), [1, 3]);
    assert_phase(&first, 1, Phase::Running { remaining: 2.0 });
    assert!(running(&first.apply(Command::Tick(2.0))).is_empty());
}

#[test]
fn group_ends_once_the_cues_it_holds_have_been_stopped() {
    let stopped = workspace(vec![group(1), within(1, audio(2)), within(1, audio(3))])
        .apply(GO)
        .apply(Command::Tick(1.0))
        .apply(Command::Stop(CueId(2)));
    assert_eq!(running(&stopped), [1, 3]);
    assert!(running(&stopped.apply(Command::Stop(CueId(3)))).is_empty());
}

#[test]
fn group_with_no_cue_that_plays_ends_as_its_action_begins() {
    let unplayable = Cue::new(CueId(2), "2", Kind::Audio, "Sound").build();
    let waiting = pre_waiting(3.0, group(1));
    let fired = workspace(vec![waiting, within(1, unplayable)]).apply(GO);
    assert_eq!(running(&fired), [1]);
    assert!(running(&fired.apply(Command::Tick(3.0))).is_empty());
}

#[test]
fn group_with_a_loop_runs_without_end_and_is_shown_so() {
    let following = then(1, Advance::Follow, 0.0, 0.0);
    let workspace = workspace(vec![
        following,
        within(1, vamp(2)),
        within(1, audio(3)),
        audio(4),
    ]);
    // No auto-follow where a cue in the group never ends
    let chain: Vec<u32> = workspace.lists()[0]
        .chain(CueId(1))
        .map(|(cue, _)| cue.id.0)
        .collect();
    assert_eq!(chain, [1, 2, 3]);
    let later = workspace.apply(GO).apply(Command::Tick(100.0));
    assert_eq!(running(&later), [1, 2]);
    assert_phase(&later, 1, Phase::Endless);
    assert_eq!(later.active_of(CueId(1)).unwrap().progress(), 1.0);
    assert_eq!(standby(&later), Some(4));
}

#[test]
fn group_in_a_group_runs_until_every_cue_at_any_depth_has_ended() {
    let inner = pre_waiting(2.0, within(1, group(2)));
    let fired = workspace(vec![
        then(1, Advance::Manual, 1.0, 0.0),
        inner,
        within(2, audio(3)),
        within(
            1,
            Cue {
                end: 2.0.into(),
                ..audio(4)
            },
        ),
    ])
    .apply(GO);
    assert_phase(&fired, 2, Phase::Scheduled { starts_in: 1.0 });
    assert_phase(&fired, 3, Phase::Scheduled { starts_in: 3.0 });
    assert_phase(&fired, 4, Phase::Scheduled { starts_in: 1.0 });
    let begun = fired.apply(Command::Tick(1.0));
    assert_phase(&begun, 1, Phase::Running { remaining: 12.0 });
    assert_phase(&begun, 2, Phase::PreWait { remaining: 2.0 });
    let inside = begun.apply(Command::Tick(2.0));
    assert_phase(&inside, 2, Phase::Running { remaining: 10.0 });
    assert_eq!(running(&inside), [1, 2, 3]);
    assert_eq!(
        running(&inside.clone().apply(Command::Tick(9.0))),
        [1, 2, 3]
    );
    assert!(running(&inside.apply(Command::Tick(10.0))).is_empty());
}
