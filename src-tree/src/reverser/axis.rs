// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Scroll axes, and selections of them.

use std::fmt;

use clap::ValueEnum;
use serde::Deserialize;

/// Scroll axis
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Vertical,
    Horizontal,
}

impl Axis {
    /// Both axes, vertical first
    pub const ALL: [Self; 2] = [Self::Vertical, Self::Horizontal];
}

impl fmt::Display for Axis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
        })
    }
}

/// Axes along which scrolling is reversed
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Axes {
    /// Vertical scrolling only
    Vertical,
    /// Horizontal scrolling only
    Horizontal,
    /// Both directions
    #[default]
    Both,
}

impl Axes {
    /// Returns whether scrolling along `axis` is reversed.
    pub fn contains(self, axis: Axis) -> bool {
        match self {
            Self::Vertical => axis == Axis::Vertical,
            Self::Horizontal => axis == Axis::Horizontal,
            Self::Both => true,
        }
    }

    /// Names the selection for log messages.
    pub fn summary(self) -> &'static str {
        match self {
            Self::Vertical => "the vertical axis",
            Self::Horizontal => "the horizontal axis",
            Self::Both => "both axes",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selections_contain_their_axes() {
        let contained = |axes: Axes| {
            Axis::ALL
                .into_iter()
                .filter(|&axis| axes.contains(axis))
                .collect::<Vec<_>>()
        };
        assert_eq!(contained(Axes::Vertical), [Axis::Vertical]);
        assert_eq!(contained(Axes::Horizontal), [Axis::Horizontal]);
        assert_eq!(contained(Axes::Both), Axis::ALL);
    }
}
