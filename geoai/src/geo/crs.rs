// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt;

/// Coordinate reference system identified by its EPSG code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crs {
    Geographic(u16),
    Projected(u16),
}

impl Crs {
    pub const WGS84: Self = Self::Geographic(4326);

    pub fn epsg(self) -> u16 {
        match self {
            Self::Geographic(code) | Self::Projected(code) => code,
        }
    }
}

impl fmt::Display for Crs {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "EPSG:{}", self.epsg())
    }
}
