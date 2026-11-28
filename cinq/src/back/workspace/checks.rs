// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Checks of the show, which only read the workspace and report, so that the
//! front end can warn the operator before a cue is fired

mod problems;
mod stranded;
mod unlit;

pub use unlit::Unlit;
