// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt;

use super::Kind;

/// Named input of a hazard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input {
    pub name: String,
    pub kind: Kind,
    /// Whether the input may be left unbound.
    pub optional: bool,
}

/// Form of a usage line: the name, with its kind unless that is a raster, and
/// bracketed if optional, as in `pga`, `[land]`, or `threshold:number`.
impl fmt::Display for Input {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let usage = match self.kind {
            Kind::Raster => self.name.clone(),
            kind => format!("{}:{kind}", self.name),
        };
        if self.optional {
            write!(formatter, "[{usage}]")
        } else {
            formatter.write_str(&usage)
        }
    }
}
