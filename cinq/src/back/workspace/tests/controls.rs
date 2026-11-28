// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Builders of the cues that act on other cues: start, pause, `GoTo`, stop,
//! devamp, and fade cues

use super::*;

pub(super) fn control(id: u32, kind: Kind, target: u32) -> Cue {
    Cue::new(CueId(id), id.to_string(), kind, "Control")
        .target(CueId(target))
        .build()
}

pub(in crate::back) fn start_cue(id: u32, target: u32) -> Cue {
    control(id, Kind::Start, target)
}

pub(super) fn pause_cue(id: u32, target: u32) -> Cue {
    control(id, Kind::Pause, target)
}

pub(super) fn goto_cue(id: u32, target: u32) -> Cue {
    control(id, Kind::GoTo, target)
}

pub(in crate::back) fn devamp_cue(id: u32, target: u32) -> Cue {
    control(id, Kind::Devamp, target)
}

pub(super) fn stop_cue(id: u32, target: u32) -> Cue {
    control(id, Kind::Stop, target)
}

/// Returns a fade cue that fades cue 1 out over four seconds and stops it
pub(super) fn fade_out(id: u32) -> Cue {
    Cue::new(CueId(id), id.to_string(), Kind::Fade, "Fade")
        .target(CueId(1))
        .duration(4.0)
        .build()
}

/// Returns a fade cue that fades cue 1 to -18 dB and leaves it playing
pub(super) fn fade_under(id: u32) -> Cue {
    Cue::new(CueId(id), id.to_string(), Kind::Fade, "Under")
        .target(CueId(1))
        .level(-18.0)
        .stop_target(false)
        .build()
}
