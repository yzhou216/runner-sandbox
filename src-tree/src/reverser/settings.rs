// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Partial settings from a single source.

use serde::Deserialize;

use super::{Axes, Devices};

/// Settings from one source, each keeping the earlier value when missing
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub device: Option<Devices>,
    pub axis: Option<Axes>,
    pub verbose: Option<bool>,
}
