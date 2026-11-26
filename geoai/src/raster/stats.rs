// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::Raster;

#[derive(Clone, Copy, Debug)]
pub struct Stats {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
}

impl<T: Copy + PartialEq + Into<f64>> Raster<T> {
    /// Statistics over the valid samples, or `None` if there are none.
    pub fn stats(&self) -> Option<Stats> {
        let (count, min, max, sum) = self.valid_values().map(Into::<f64>::into).fold(
            (0, f64::INFINITY, f64::NEG_INFINITY, 0.0),
            |(count, min, max, sum), value| {
                (count + 1, min.min(value), max.max(value), sum + value)
            },
        );
        (count > 0).then(|| Stats {
            count,
            min,
            max,
            mean: sum / count as f64,
        })
    }
}
