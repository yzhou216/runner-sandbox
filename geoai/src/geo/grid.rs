// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

mod index;
#[cfg(test)]
mod tests;

use super::{Bounds, Crs};

/// North-up grid of `width` by `height` pixels.
///
/// The origin is the upper-left corner of the upper-left pixel, matching the
/// GeoTIFF `PixelIsArea` convention.  Rows run from north to south.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    width: u32,
    height: u32,
    origin: (f64, f64),
    pixel_size: (f64, f64),
    crs: Option<Crs>,
}

impl Grid {
    /// Create a grid with its origin at `(0, 0)`, unit pixels, and no CRS.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            origin: (0.0, 0.0),
            pixel_size: (1.0, 1.0),
            crs: None,
        }
    }

    pub fn with_origin(self, x: f64, y: f64) -> Self {
        Self {
            origin: (x, y),
            ..self
        }
    }

    pub fn with_pixel_size(self, width: f64, height: f64) -> Self {
        Self {
            pixel_size: (width, height),
            ..self
        }
    }

    pub fn with_crs(self, crs: impl Into<Option<Crs>>) -> Self {
        Self {
            crs: crs.into(),
            ..self
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn origin(&self) -> (f64, f64) {
        self.origin
    }

    pub fn pixel_size(&self) -> (f64, f64) {
        self.pixel_size
    }

    pub fn crs(&self) -> Option<Crs> {
        self.crs
    }

    pub fn len(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn bounds(&self) -> Bounds {
        let ((west, north), (pixel_width, pixel_height)) = (self.origin, self.pixel_size);
        Bounds {
            west,
            south: north - f64::from(self.height) * pixel_height,
            east: west + f64::from(self.width) * pixel_width,
            north,
        }
    }
}
