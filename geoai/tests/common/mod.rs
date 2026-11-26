// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Helpers for the tests of hazards written in Scheme.

use std::sync::Arc;

use anyhow::{Result, ensure};
use geoai::{
    geo::Grid,
    hazard::{Hazard, Value},
    raster::Raster,
    script,
};

/// Single hazard defined by `source`.
pub fn hazard(source: &str) -> Result<Hazard> {
    let mut hazards = script::load(source, "test.scm")?;
    ensure!(
        hazards.len() == 1,
        "expected one hazard, found {}",
        hazards.len()
    );
    Ok(hazards.remove(0))
}

/// Raster of a single row of pixels holding `values`, with NaN marking a
/// missing value.
pub fn row(values: &[f32]) -> Result<Value> {
    let raster = Raster::new(Grid::new(values.len() as u32, 1), values.to_vec())?;
    Ok(Value::Raster(Arc::new(raster)))
}
