// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of copying, cutting, and pasting cues

mod basic;
mod elsewhere;
mod limits;
mod targets;

use super::{operators::rows, *};
use crate::{
    back::{Command, Cue, InstrumentId, Kind, OutputEdit, OutputId, Sends},
    front::update::editing::Clipboard,
};
