// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Built-in earthquake hazard.

use std::{collections::HashMap, path::Path, sync::Arc};

use anyhow::{Context, Result};
use geoai::{
    catalog::Catalog,
    format,
    geo::Grid,
    hazard::{Hazard, Value},
    raster::Raster,
    storage::Location,
};
use itertools::Itertools;

const NODATA: f32 = -9999.0;

/// Two pixels without data, then values on either side of each class boundary.
const PGA: [f32; 7] = [NODATA, NODATA, 49.9, 50.0, 149.9, 150.0, 300.0];

/// Result of `compute` given the built-in earthquake hazard.
fn with_earthquake<T>(compute: impl FnOnce(&Hazard) -> Result<T>) -> Result<T> {
    let catalog = Catalog::builtin()?;
    compute(
        catalog
            .get("earthquake")
            .context("no built-in earthquake hazard")?,
    )
}

/// Classes along a row of pixels with the values of [`PGA`], given an
/// optional land mask.
fn classes(land: Option<Value>) -> Result<Vec<u8>> {
    let grid = Grid::new(PGA.len() as u32, 1);
    let pga = Raster::new(grid, PGA.to_vec())?.with_nodata(NODATA);
    let values = [Some(Value::Raster(Arc::new(pga))), land];
    with_earthquake(|earthquake| Ok(earthquake.evaluate::<u8>(grid, &values)?.data().to_vec()))
}

#[test]
fn classes_follow_pga_thresholds() -> Result<()> {
    assert_eq!(classes(None)?, [0, 0, 2, 3, 3, 4, 5]);
    Ok(())
}

#[test]
fn land_without_pga_is_class_one() -> Result<()> {
    let land = Raster::new(Grid::new(7, 1), vec![0_u8, 1, 1, 1, 1, 1, 1])?.with_nodata(0);
    assert_eq!(
        classes(Some(Value::Raster(Arc::new(land))))?,
        [0, 1, 2, 3, 3, 4, 5]
    );
    Ok(())
}

/// The reference output has tiles the tiff crate cannot decode, so this
/// compares the number of pixels in each class.  Without a land mask, the
/// reference's class 1 pixels are class 0.
#[test]
fn reproduces_the_reference_classes() -> Result<()> {
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data");
    let pga = format::read_layer(&Location::File(data.join("PGA_250y.tif")))?;
    let grid = format::read_grid(&Location::File(data.join("hazard_earthquake.tif")))?;
    let classes = with_earthquake(|earthquake| {
        earthquake.evaluate::<u8>(grid, &[Some(Value::Raster(pga)), None])
    })?;
    let counts: HashMap<u8, usize> = classes.data().iter().copied().counts();
    assert_eq!(
        counts,
        HashMap::from([
            (0, 58_429_720 + 8_696_400),
            (2, 61_951),
            (3, 8_283_598),
            (4, 2_995_352),
            (5, 76_518),
        ])
    );
    Ok(())
}
