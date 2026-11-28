// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how a cue that plays is told by its time and level

use super::*;
use crate::{
    back::{SILENCE, parse_level},
    front::view::format::decibels,
};

#[test]
fn a_run_moved_below_silence_is_told_at_silence() {
    let cue = |id: u32, kind, level: f32| {
        Cue::new(CueId(id), id.to_string(), kind, "Rain")
            .advance(Advance::Continue)
            .file("sound.ogg")
            .target(CueId(1))
            .level(level)
            .duration(1.0)
            .stop_target(false)
            .build()
    };
    let workspace = fresh(vec![cue(1, Kind::Audio, 0.0), cue(2, Kind::Fade, -60.0)]);
    let loaded = loaded(1, "sound.ogg", sound(10.0));
    let ducked = [loaded, Command::Go { list: 0 }, Command::Tick(2.0)]
        .into_iter()
        .fold(workspace, Workspace::apply)
        .apply(Command::Edit(CueId(1), Edit::Level(-30.0)));
    let active = ducked.active_of(CueId(1)).expect("the cue plays");
    assert_eq!(active.level, -90.0);
    let cue = ducked.cue(CueId(1)).expect("the cue is there");
    assert_eq!(activity(active, cue), "0:08.0 left, -inf dB");
}

#[test]
fn silence_reads_as_it_is_typed() {
    assert_eq!(
        [SILENCE, -59.9, 0.0, 12.0].map(decibels),
        ["-inf", "-59.9", "+0.0", "+12.0"]
    );
    assert_eq!(parse_level(&decibels(SILENCE)), Some(SILENCE));
}

#[test]
fn a_devamped_run_is_told_by_the_time_it_has_left_instead_of_looping() {
    let rain = Cue::new(CueId(1), "1", Kind::Audio, "Rain")
        .file("sound.ogg")
        .looping(true)
        .loop_start(2.0)
        .loop_end(6.0)
        .build();
    let devamp = Cue::new(CueId(2), "2", Kind::Devamp, "Leave rain")
        .target(CueId(1))
        .build();
    let workspace = [
        loaded(1, "sound.ogg", sound(10.0)),
        Command::Go { list: 0 },
        Command::Tick(1.0),
    ]
    .into_iter()
    .fold(fresh(vec![rain, devamp]), Workspace::apply);
    let told = |workspace: &Workspace| {
        let cue = workspace.cue(CueId(1)).expect("the cue is there");
        activity(workspace.active_of(CueId(1)).expect("the cue plays"), cue)
    };
    assert_eq!(told(&workspace), "looping, +0.0 dB");
    let devamped = [Command::Go { list: 0 }, Command::Tick(1.0)]
        .into_iter()
        .fold(workspace, Workspace::apply);
    assert_eq!(told(&devamped), "0:08.0 left, +0.0 dB");
}

#[test]
fn a_wait_ahead_is_told_in_the_figures_of_the_time_left() {
    let waiting = [
        Command::Edit(CueId(1), Edit::PreWait(16.1.into())),
        Command::Go { list: 0 },
    ]
    .into_iter()
    .fold(fresh(vec![memo(1)]), Workspace::apply);
    let active = waiting.active_of(CueId(1)).expect("the cue waits");
    assert_eq!(phase(active), "in 0:16.1");
}
