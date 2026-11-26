// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::{self, Write};

use anyhow::Result;
use tabwriter::TabWriter;

use crate::{format, raster::Raster, storage::Location};

#[derive(clap::Args)]
pub struct Args {
    /// GeoTIFF to describe, a path or a URL such as s3://bucket/flood.tif
    #[arg(value_parser = super::location)]
    file: Location,
    /// Also print the value at these coordinates, in the CRS of the raster
    #[arg(long, num_args = 2, value_names = ["X", "Y"], allow_negative_numbers = true)]
    at: Option<Vec<f64>>,
}

pub fn run(args: &Args) -> Result<()> {
    let raster: Raster<f32> = format::read(&args.file)?;
    let mut out = TabWriter::new(io::stdout().lock());
    summary(&raster, args.at.as_deref().map(|at| (at[0], at[1])))
        .try_for_each(|(label, value)| writeln!(out, "{label}:\t{value}"))?;
    out.flush()?;
    Ok(())
}

fn summary(
    raster: &Raster<f32>,
    point: Option<(f64, f64)>,
) -> impl Iterator<Item = (String, String)> + '_ {
    let grid = raster.grid();
    let bounds = grid.bounds();
    let (pixel_width, pixel_height) = grid.pixel_size();
    [
        ("size", format!("{} x {}", grid.width(), grid.height())),
        (
            "crs",
            grid.crs().map_or("unknown".into(), |crs| crs.to_string()),
        ),
        ("pixel size", format!("{pixel_width} x {pixel_height}")),
        (
            "bounds",
            format!(
                "[{}, {}] - [{}, {}]",
                bounds.west, bounds.south, bounds.east, bounds.north
            ),
        ),
        ("nodata", text(raster.nodata())),
        (
            "stats",
            raster.stats().map_or("no valid samples".into(), |stats| {
                format!(
                    "count={} min={:.6} max={:.6} mean={:.6}",
                    stats.count, stats.min, stats.max, stats.mean
                )
            }),
        ),
    ]
    .into_iter()
    .map(|(label, value)| (label.to_string(), value))
    .chain(point.map(|(x, y)| (format!("value at ({x}, {y})"), text(raster.get(x, y)))))
}

/// `value` as text, or `none` if it is missing.
fn text(value: Option<f32>) -> String {
    value.map_or("none".into(), |value| value.to_string())
}
