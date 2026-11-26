// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{fmt, str::FromStr};

use anyhow::{Error, Result, bail};

/// Sample type of the raster a hazard produces.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SampleType {
    U8,
    #[default]
    F32,
}

impl SampleType {
    /// Whether `value` can be a sample of this type: a whole number from 0 to
    /// 255 for u8, or for f32 NaN, an infinity, or any number that rounds to a
    /// finite f32.
    pub fn can_hold(self, value: f64) -> bool {
        match self {
            Self::U8 => value.fract() == 0.0 && (0.0..=255.0).contains(&value),
            Self::F32 => !value.is_finite() || (value as f32).is_finite(),
        }
    }
}

impl fmt::Display for SampleType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::U8 => "u8",
            Self::F32 => "f32",
        })
    }
}

impl FromStr for SampleType {
    type Err = Error;

    fn from_str(name: &str) -> Result<Self> {
        match name {
            "u8" => Ok(Self::U8),
            "f32" => Ok(Self::F32),
            _ => bail!("unknown sample type `{name}`; expected u8 or f32"),
        }
    }
}
