// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the patch, light cues, and the DMX that their levels make

mod levels;
mod patch;
mod show_file;

use super::*;
use crate::back::{Instrument, InstrumentId};

/// Returns a light cue that fades instruments to `levels` over four seconds
fn light(id: u32, levels: &[(u32, f32)]) -> Cue {
    Cue::new(CueId(id), id.to_string(), Kind::Light, "Light")
        .duration(4.0)
        .lights(
            levels
                .iter()
                .map(|&(instrument, level)| (InstrumentId(instrument), level))
                .collect(),
        )
        .build()
}

/// Returns a workspace of `cues` with instruments 1 and 2 at addresses 1
/// and 2
fn lit(cues: Vec<Cue>) -> Workspace {
    let instrument = |id, address| Instrument {
        id: InstrumentId(id),
        name: id.to_string(),
        universe: 1,
        address,
    };
    workspace(cues).with_patch(vec![instrument(1, 1), instrument(2, 2)])
}

fn level(workspace: &Workspace, instrument: u32) -> f32 {
    workspace.light_level(InstrumentId(instrument))
}

fn tick(seconds: f64) -> Command {
    Command::Tick(seconds)
}
