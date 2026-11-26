// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::Grid;

impl Grid {
    /// Model coordinates of the center of the pixel at row-major `index`.
    pub fn pixel_center(&self, index: usize) -> (f64, f64) {
        let columns = self.width as usize;
        let (column, row) = ((index % columns) as f64, (index / columns) as f64);
        let ((west, north), (pixel_width, pixel_height)) = (self.origin, self.pixel_size);
        (
            west + (column + 0.5) * pixel_width,
            north - (row + 0.5) * pixel_height,
        )
    }

    /// Row-major index of the pixel containing `(x, y)`, if inside the grid.
    ///
    /// A point on the border of two pixels belongs to the later one, even
    /// where rounding places it slightly before the border.
    pub fn index_of(&self, x: f64, y: f64) -> Option<usize> {
        let ((west, north), (pixel_width, pixel_height)) = (self.origin, self.pixel_size);
        let column = whole_pixels(x - west, pixel_width, x.abs().max(west.abs()));
        let row = whole_pixels(north - y, pixel_height, y.abs().max(north.abs()));
        ((0.0..f64::from(self.width)).contains(&column)
            && (0.0..f64::from(self.height)).contains(&row))
        .then(|| row as usize * self.width as usize + column as usize)
    }
}

/// Whole pixels of `size` in `distance`, a difference between two coordinates
/// whose absolute values are at most `magnitude`.  A distance that falls short
/// of a whole number of pixels by no more than a rounding error counts as that
/// number.
fn whole_pixels(distance: f64, size: f64, magnitude: f64) -> f64 {
    // Rounding error of such coordinates, in pixels, but at least that of a
    // point computed from the origin of a grid millions of pixels away
    let tolerance = (16.0 * f64::EPSILON * magnitude / size).max(1e-8);
    (distance / size + tolerance).floor()
}
