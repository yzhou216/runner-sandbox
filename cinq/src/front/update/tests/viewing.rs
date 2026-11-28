// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of scrolling a row at a time, selecting the rows in view, and
//! putting the selected row in view

mod in_view;
mod placing;
mod rounding;
mod scrolling;

use super::{show, typing_task};
use crate::{back::CueId, front::App};

/// Returns the state after the keys are pressed from a show with the second
/// cue selected, and the tasks that the last one started
fn pressed(keys: &str) -> (App, usize) {
    let (app, task) = typing_task(show(Some(CueId(2))), keys);
    (app, task.units())
}
