// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Values and procedures that Rust provides to hazard files.

use std::{fmt, sync::Arc};

use anyhow::{Context, Result, anyhow, bail, ensure};
use steel::{
    SteelErr, SteelVal,
    rerrs::ErrorKind,
    rvals::{Custom, FromSteelVal, IntoSteelVal},
    steel_vm::{engine::Engine, register_fn::RegisterFn},
};

use super::conversion;
use crate::{
    geo::{Constant, Layer},
    hazard::Value,
    pixelwise::Pixelwise,
    sexp::Sexp,
};

/// Raster as a Scheme value, opaque to Scheme code.
#[derive(Clone)]
struct RasterValue(Arc<dyn Layer>);

impl Custom for RasterValue {
    fn fmt(&self) -> Option<Result<String, fmt::Error>> {
        Some(Ok("#<raster>".to_owned()))
    }
}

/// Make rasters and the procedures of this module available in `engine`.
pub fn register(engine: &mut Engine) {
    engine.register_type::<RasterValue>("raster?");
    engine.register_fn(
        "%pixelwise",
        |values: Vec<SteelVal>, names: SteelVal, expression: SteelVal| {
            pixelwise(&values, &names, &expression).map_err(scheme_error)
        },
    );
}

/// Scheme value of an input value, or `#f` for an unbound input.
pub fn to_scheme(value: Option<&Value>) -> Result<SteelVal> {
    Ok(match value {
        Some(Value::Raster(layer)) => RasterValue(Arc::clone(layer))
            .into_steelval()
            .map_err(|error| anyhow!("{error}"))?,
        Some(Value::Number(number)) => SteelVal::NumV(*number),
        None => SteelVal::BoolV(false),
    })
}

/// Layer of `value`, if it is a raster.
pub fn layer(value: &SteelVal) -> Option<Arc<dyn Layer>> {
    RasterValue::from_steelval(value)
        .ok()
        .map(|raster| raster.0)
}

/// Implementation of `pixelwise`: raster computed by `expression` from the
/// values bound to `names`, which the macro passes in the same order.
fn pixelwise(values: &[SteelVal], names: &SteelVal, expression: &SteelVal) -> Result<RasterValue> {
    let Sexp::List(names) = Sexp::try_from(names)? else {
        bail!("expected a list of names, found {names}");
    };
    let bindings = names
        .iter()
        .zip(values)
        .map(|(name, value)| {
            let name = name
                .as_symbol()
                .with_context(|| format!("expected a name to bind, found {name}"))?;
            let layer = binding(value).with_context(|| format!("cannot bind `{name}`"))?;
            Ok((name.to_owned(), layer))
        })
        .collect::<Result<_>>()?;
    let layer = Pixelwise::new(&Sexp::try_from(expression)?, bindings)?;
    Ok(RasterValue(Arc::new(layer)))
}

/// Layer that a name bound by `pixelwise` stands for: that of a raster, the
/// same finite number everywhere, or `None` for `#f`, which is missing
/// everywhere.
fn binding(value: &SteelVal) -> Result<Option<Arc<dyn Layer>>> {
    if let SteelVal::BoolV(false) = value {
        return Ok(None);
    }
    let layer: Arc<dyn Layer> = match conversion::number(value) {
        Some(number) => {
            ensure!(
                number.is_finite(),
                "expected a finite number, found {value}"
            );
            Arc::new(Constant(number))
        }
        None => layer(value)
            .with_context(|| format!("expected a raster, a number, or #f, found {value}"))?,
    };
    Ok(Some(layer))
}

/// Scheme error with the message of `error`, followed by its causes.
fn scheme_error(error: anyhow::Error) -> SteelErr {
    SteelErr::new(ErrorKind::Generic, format!("{error:#}"))
}
