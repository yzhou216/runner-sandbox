// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the playback clock: ticks that come out of order, and the clock
//! once the output stops the last running cue

use std::time::{Duration, Instant};

use super::show;
use crate::{
    back::{Active, CueId, Media, Phase, Workspace, loaded, sound},
    front::{App, Message},
};

#[test]
fn ticks_out_of_order_count_each_moment_once_and_never_turn_the_clock_back() {
    let app = show(Some(CueId(1)));
    let workspace = app.workspace.apply(loaded(1, "sound.ogg", sound(10.0)));
    let (playing, _) = App { workspace, ..app }.handled(Message::Go);
    let start = Instant::now();
    let at = |milliseconds| start + Duration::from_millis(milliseconds);
    let (ticked, _) = App {
        last_tick: Some(start),
        ..playing
    }
    .next(Message::Tick(at(50)));
    let (late, _) = ticked.next(Message::Tick(at(20)));
    assert_eq!(late.last_tick, Some(at(50)));
    let (ticked, _) = late.next(Message::Tick(at(75)));
    assert_eq!(ticked.last_tick, Some(at(75)));
    let phase = ticked.workspace.active_of(CueId(1)).map(Active::phase);
    let Some(Phase::Running { remaining }) = phase else {
        panic!("the cue runs, but is {phase:?}");
    };
    assert!(
        (remaining - 9.925).abs() < 1e-9,
        "{remaining} seconds remain"
    );
}

#[test]
fn clock_stops_once_the_output_cannot_play_the_last_running_cue() {
    let app = show(Some(CueId(1)));
    let workspace = [
        loaded(1, "sound.ogg", sound(10.0)),
        loaded(2, "sound.ogg", sound(10.0)),
    ]
    .into_iter()
    .fold(app.workspace, Workspace::apply);
    let (playing, _) = App { workspace, ..app }.next(Message::Go);
    let playing = playing.clocked();
    assert!(playing.last_tick.is_some());
    // Run that the sound output cannot start, which it stops in the same
    // update
    let failed = loaded(
        1,
        "sound.ogg",
        Media::Failed("the device has gone".to_owned()),
    );
    let stopped = App {
        workspace: playing.workspace.apply(failed),
        ..playing
    }
    .clocked();
    assert_eq!(stopped.last_tick, None);
    let before = Instant::now();
    let (gone, _) = stopped.next(Message::Go);
    let started = gone.clocked().last_tick;
    assert!(started.is_some_and(|start| start >= before), "{started:?}");
}
