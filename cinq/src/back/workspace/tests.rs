// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the rules that a workspace applies, whose shared builders are in
//! `fixtures` and `cues`

mod adding;
mod collapsed_editing;
mod collapsing;
mod controls;
mod cues;
mod devamp_cues;
mod devamping;
mod editing;
mod files;
mod fixtures;
mod going;
mod group_control;
mod grouping;
mod groups_adding;
mod groups_deleting;
mod groups_moving;
mod groups_pasting;
mod groups_playing;
mod history;
mod holding;
mod levels;
mod light;
mod loop_progress;
mod network;
mod output_pasting;
mod outputs;
mod pasting;
mod picture;
mod playback;
mod releases;
mod remote;
mod roundtrip;
mod show;
mod starting;
mod stopping;
mod video;

use std::path::Path;

use controls::{control, fade_out, fade_under, goto_cue, pause_cue, stop_cue};
pub(in crate::back) use controls::{devamp_cue, start_cue};
pub(crate) use cues::*;
pub(crate) use fixtures::*;

use super::{Workspace, running::playback::PANIC_FADE};
use crate::back::{
    Advance, Command, Cue, CueId, CueList, Edit, Kind, Media, Phase, Place, Problem, SILENCE,
};
