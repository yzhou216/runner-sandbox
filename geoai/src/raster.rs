// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

mod layer;
mod stats;

pub use stats::Stats;

use anyhow::{Result, ensure};

use crate::geo::Grid;

/// Single-band raster held in memory.
#[derive(Clone, Debug)]
pub struct Raster<T> {
    grid: Grid,
    data: Vec<T>,
    nodata: Option<T>,
}

impl<T: Copy + PartialEq> Raster<T> {
    /// Wrap row-major `data`, which must hold one sample per pixel of `grid`.
    pub fn new(grid: Grid, data: Vec<T>) -> Result<Self> {
        ensure!(
            data.len() == grid.len(),
            "raster has {} samples but {} pixels",
            data.len(),
            grid.len()
        );
        Ok(Self {
            grid,
            data,
            nodata: None,
        })
    }

    pub fn with_nodata(self, nodata: impl Into<Option<T>>) -> Self {
        Self {
            nodata: nodata.into(),
            ..self
        }
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    /// Row-major samples, including nodata.
    pub fn data(&self) -> &[T] {
        &self.data
    }

    pub fn nodata(&self) -> Option<T> {
        self.nodata
    }
}

impl<T: Copy + PartialEq + Into<f64>> Raster<T> {
    /// Value of the pixel containing `(x, y)`, unless that is outside the grid
    /// or not valid.
    pub fn get(&self, x: f64, y: f64) -> Option<T> {
        self.grid
            .index_of(x, y)
            .map(|index| self.data[index])
            .filter(|&value| self.is_valid(value))
    }

    fn valid_values(&self) -> impl Iterator<Item = T> + '_ {
        self.data
            .iter()
            .copied()
            .filter(|&value| self.is_valid(value))
    }

    /// Whether `value` is data, which it is unless it is the nodata value or
    /// not a finite number.
    fn is_valid(&self, value: T) -> bool {
        let number: f64 = value.into();
        self.nodata != Some(value) && number.is_finite()
    }
}
