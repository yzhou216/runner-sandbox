// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cues that run: the cues that GO has fired in [`active`], and the rules
//! that start, time, and stop them

pub(super) mod active;
mod control;
mod group_runs;
pub(super) mod lighting;
mod networking;
pub(super) mod playback;
mod stopping;
