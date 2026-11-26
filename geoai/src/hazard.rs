// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Hazards: rasters computed from named inputs.
//!
//! A hazard declares its name, its inputs, and the raster it produces, and
//! leaves the computation to its [`Body`].  It knows neither where the values
//! of its inputs come from nor the language its body is written in.

mod body;
mod definition;
mod input;
mod kind;
mod sample_type;
mod value;

pub use body::Body;
pub use input::Input;
pub use kind::Kind;
pub use sample_type::SampleType;
pub use value::Value;

use std::{any, fmt};

use anyhow::{Context, Result, bail, ensure};
use itertools::Itertools;
use num_traits::FromPrimitive;

use crate::{geo::Grid, raster::Raster};

pub struct Hazard {
    name: String,
    inputs: Vec<Input>,
    sample_type: SampleType,
    description: Option<String>,
    nodata: Option<f64>,
    body: Box<dyn Body>,
}

impl Hazard {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    pub fn sample_type(&self) -> SampleType {
        self.sample_type
    }

    /// Band description of the output, the hazard name unless declared.
    pub fn description(&self) -> &str {
        self.description.as_deref().unwrap_or(&self.name)
    }

    /// Order `bindings` of input names to values by the inputs of this hazard.
    ///
    /// Unknown and repeated names are errors, as are unbound required inputs.
    pub fn bind<'a, T>(&self, bindings: &'a [(String, T)]) -> Result<Vec<Option<&'a T>>> {
        if let Some((name, _)) = bindings.iter().find(|(name, _)| !self.has_input(name)) {
            bail!("hazard `{}` has no input `{name}`", self.name);
        }
        if let Some(name) = bindings.iter().map(|(name, _)| name).duplicates().next() {
            bail!("input `{name}` is bound more than once");
        }
        self.inputs
            .iter()
            .map(|input| {
                let value = bindings
                    .iter()
                    .find(|(name, _)| *name == input.name)
                    .map(|(_, value)| value);
                ensure!(
                    value.is_some() || input.optional,
                    "hazard `{}` requires input `{}`",
                    self.name,
                    input.name
                );
                Ok(value)
            })
            .collect()
    }

    /// Compute the hazard at the center of every pixel of `grid`.
    ///
    /// `values` holds one value per input in order, with `None` for an unbound
    /// optional input, and its rasters must be in the CRS of `grid`.  Values
    /// are converted to `T` as [`Raster::from_layer`] describes, and a missing
    /// value is an error unless the hazard declares nodata.
    pub fn evaluate<T>(&self, grid: Grid, values: &[Option<Value>]) -> Result<Raster<T>>
    where
        T: Copy + PartialEq + Into<f64> + Send + Sync + FromPrimitive,
    {
        let evaluate = || {
            self.check(values)?;
            self.check_crs(&grid, values)?;
            let nodata = self
                .nodata
                .map(|nodata| {
                    T::from_f64(nodata).with_context(|| {
                        format!("nodata {nodata} does not fit in {}", any::type_name::<T>())
                    })
                })
                .transpose()?;
            let layer = self.body.compute(values)?;
            Raster::from_layer(grid, layer.as_ref(), nodata)
        };
        evaluate().with_context(|| format!("cannot compute hazard `{}`", self.name))
    }

    /// Ensure that `values` holds a value of the kind of each input, leaving
    /// only optional inputs unbound.
    fn check(&self, values: &[Option<Value>]) -> Result<()> {
        ensure!(
            values.len() == self.inputs.len(),
            "expected {} values, one per input, but found {}",
            self.inputs.len(),
            values.len()
        );
        self.inputs
            .iter()
            .zip(values)
            .try_for_each(|(input, value)| {
                match value {
                    Some(value) => ensure!(
                        value.kind() == input.kind,
                        "input `{}` takes a {}, not a {}",
                        input.name,
                        input.kind,
                        value.kind()
                    ),
                    None => ensure!(input.optional, "input `{}` is required", input.name),
                }
                Ok(())
            })
    }

    /// Ensure that every raster in `values` is in the CRS of `grid`, where
    /// both are known, since nothing reprojects rasters yet.
    fn check_crs(&self, grid: &Grid, values: &[Option<Value>]) -> Result<()> {
        let Some(expected) = grid.crs() else {
            return Ok(());
        };
        self.inputs
            .iter()
            .zip(values)
            .try_for_each(|(input, value)| {
                let crs = match value {
                    Some(Value::Raster(layer)) => layer.crs(),
                    _ => None,
                };
                match crs {
                    Some(crs) if crs != expected => bail!(
                        "input `{}` is in {crs}, but the output is in {expected}; \
                         reprojection is not supported yet",
                        input.name
                    ),
                    _ => Ok(()),
                }
            })
    }

    fn has_input(&self, name: &str) -> bool {
        self.inputs.iter().any(|input| input.name == name)
    }
}

/// Everything but the body, which need not be printable.
impl fmt::Debug for Hazard {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Hazard")
            .field("name", &self.name)
            .field("inputs", &self.inputs)
            .field("sample_type", &self.sample_type)
            .field("description", &self.description)
            .field("nodata", &self.nodata)
            .finish_non_exhaustive()
    }
}
