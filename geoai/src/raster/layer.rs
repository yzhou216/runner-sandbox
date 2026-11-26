// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::any;

use anyhow::{Context, Result};
use num_traits::FromPrimitive;
use rayon::prelude::*;

use super::Raster;
use crate::geo::{Crs, Grid, Layer};

impl<T: Copy + PartialEq + Into<f64> + Send + Sync> Layer for Raster<T> {
    fn sample(&self, x: f64, y: f64) -> Option<f64> {
        self.get(x, y).map(Into::into)
    }

    fn crs(&self) -> Option<Crs> {
        self.grid.crs()
    }
}

impl<T: Copy + PartialEq + Into<f64> + Send + Sync + FromPrimitive> Raster<T> {
    /// Sample `layer` at the center of every pixel of `grid` in parallel.
    ///
    /// Values are converted to `T`, losing their fractional part if `T` is an
    /// integer type, and `nodata` is written where `layer` has no data or a
    /// value that is not a finite number.  A value outside the range of `T` is
    /// an error, as is a missing value without `nodata`.
    pub fn from_layer(grid: Grid, layer: &dyn Layer, nodata: Option<T>) -> Result<Self> {
        let data = (0..grid.len())
            .into_par_iter()
            .map(|index| grid.pixel_center(index))
            .map(|(x, y)| {
                to_sample(layer.sample(x, y), nodata).with_context(|| format!("at ({x}, {y})"))
            })
            .collect::<Result<_>>()?;
        Ok(Self { grid, data, nodata })
    }
}

/// `value` as a sample, or `nodata` if it is missing or not a finite number.
fn to_sample<T: Copy + Into<f64> + FromPrimitive>(
    value: Option<f64>,
    nodata: Option<T>,
) -> Result<T> {
    match value.filter(|value| value.is_finite()) {
        Some(value) => T::from_f64(value)
            // Conversion to f32 overflows to infinity instead of failing
            .filter(|&sample| Into::<f64>::into(sample).is_finite())
            .with_context(|| format!("{value} does not fit in {}", any::type_name::<T>())),
        None => nodata.context("missing value, and no nodata to write instead"),
    }
}
