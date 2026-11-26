// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{fmt, str::FromStr};

use anyhow::{Error, Result, bail};

/// Kind of value an input of a hazard takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Raster,
    Number,
}

impl fmt::Display for Kind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Raster => "raster",
            Self::Number => "number",
        })
    }
}

impl FromStr for Kind {
    type Err = Error;

    fn from_str(name: &str) -> Result<Self> {
        match name {
            "raster" => Ok(Self::Raster),
            "number" => Ok(Self::Number),
            _ => bail!("unknown input kind `{name}`; expected raster or number"),
        }
    }
}
