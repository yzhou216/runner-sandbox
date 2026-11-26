// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S-expressions: Scheme values as plain data.
//!
//! Hazard signatures and `pixelwise` expressions reach Rust as
//! [S-expressions](Sexp), so the code that interprets them does not depend on
//! any Scheme implementation.

use std::fmt;

use itertools::Itertools;

/// Number, string, symbol, or list, as written in Scheme.
#[derive(Clone, Debug, PartialEq)]
pub enum Sexp {
    Number(f64),
    String(String),
    Symbol(String),
    List(Vec<Sexp>),
}

impl Sexp {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(number) => Some(*number),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(string) => Some(string),
            _ => None,
        }
    }

    pub fn as_symbol(&self) -> Option<&str> {
        match self {
            Self::Symbol(symbol) => Some(symbol),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Sexp]> {
        match self {
            Self::List(items) => Some(items),
            _ => None,
        }
    }
}

/// Scheme syntax, for error messages.
impl fmt::Display for Sexp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(number) if number.is_nan() => formatter.write_str("+nan.0"),
            Self::Number(number) if number.is_infinite() => {
                formatter.write_str(if *number > 0.0 { "+inf.0" } else { "-inf.0" })
            }
            Self::Number(number) => write!(formatter, "{number}"),
            Self::String(string) => write!(formatter, "{string:?}"),
            Self::Symbol(symbol) => formatter.write_str(symbol),
            Self::List(items) => write!(formatter, "({})", items.iter().format(" ")),
        }
    }
}
