// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::Crs;

/// Raster data queried at model coordinates, whatever its file format, grid,
/// or origin.
///
/// Rasters read from files, rasters computed pixel by pixel, and constants
/// are all layers, so any of them can feed any computation.  Layers are
/// shared across the threads that compute a raster.
pub trait Layer: Send + Sync {
    /// Value at `(x, y)`, a finite number, or `None` where the layer has no
    /// data.
    fn sample(&self, x: f64, y: f64) -> Option<f64>;

    /// CRS of the coordinates at which the layer is sampled, if known.
    fn crs(&self) -> Option<Crs> {
        None
    }
}

/// Layer with the same value everywhere, which is missing everywhere if it is
/// not a finite number.
pub struct Constant(pub f64);

impl Layer for Constant {
    fn sample(&self, _x: f64, _y: f64) -> Option<f64> {
        Some(self.0).filter(|value| value.is_finite())
    }
}
