// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Steel values as the data and numbers that the rest of geoai reads.

use anyhow::{Error, Result, bail};
use num_traits::ToPrimitive;
use steel::SteelVal;

use super::primitives;
use crate::sexp::Sexp;

impl TryFrom<&SteelVal> for Sexp {
    type Error = Error;

    fn try_from(value: &SteelVal) -> Result<Self> {
        Ok(match value {
            SteelVal::StringV(string) => Self::String(string.to_string()),
            SteelVal::SymbolV(symbol) => Self::Symbol(unmangled(symbol).to_owned()),
            SteelVal::ListV(items) => {
                Self::List(items.iter().map(Self::try_from).collect::<Result<_>>()?)
            }
            other => match number(other) {
                Some(number) => Self::Number(number),
                None if primitives::layer(other).is_some() => bail!(
                    "a raster cannot be inserted with `,`; bind it to a name in (pixelwise (name ...) ...)"
                ),
                None => bail!("expected a number, string, symbol, or list, found {other}"),
            },
        })
    }
}

/// Value of `value` if it is a real number, exact or inexact.
pub fn number(value: &SteelVal) -> Option<f64> {
    match value {
        SteelVal::IntV(integer) => integer.to_f64(),
        SteelVal::NumV(number) => Some(*number),
        SteelVal::Rational(ratio) => ratio.to_f64(),
        SteelVal::BigNum(integer) => integer.to_f64(),
        SteelVal::BigRational(ratio) => ratio.to_f64(),
        _ => None,
    }
}

/// Name of `symbol` as written.  Steel renames the identifiers of a module
/// that a file requires, writing `*` as `##mm12__%#__*`, and the renamed form
/// reaches data quoted by a macro such as `pixelwise`.
fn unmangled(symbol: &str) -> &str {
    symbol
        .strip_prefix("##mm")
        .and_then(|renamed| renamed.split_once("__%#__"))
        .map_or(symbol, |(_, name)| name)
}
