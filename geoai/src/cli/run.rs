// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Context, Result};

use super::hazards;
use crate::{
    format,
    geo::Grid,
    hazard::{Hazard, Kind, SampleType, Value},
    storage::Location,
};

#[derive(clap::Args)]
pub struct Args {
    /// Hazard to compute, as listed by `geoai list`
    hazard: String,
    /// GeoTIFF to write, a path or a URL such as s3://bucket/flood.tif
    #[arg(value_parser = super::location)]
    output: Location,
    /// Value of an input of the hazard, a raster file or a number; repeat for
    /// each input
    #[arg(long = "input", value_name = "NAME=VALUE", value_parser = binding)]
    inputs: Vec<(String, String)>,
    /// Raster whose grid the output takes, instead of the first raster input's
    #[arg(long, value_name = "FILE", value_parser = super::location)]
    like: Option<Location>,
    #[command(flatten)]
    hazards: hazards::Args,
}

pub fn run(args: &Args) -> Result<()> {
    let catalog = args.hazards.load()?;
    let hazard = catalog
        .get(&args.hazard)
        .with_context(|| format!("unknown hazard `{}`; see `geoai list`", args.hazard))?;
    format::check_writable(&args.output)?;
    let texts = hazard.bind(&args.inputs)?;
    let values = read_values(hazard, &texts)?;
    let grid = output_grid(args.like.as_ref(), hazard, &texts)?;
    let (description, output) = (hazard.description(), &args.output);
    match hazard.sample_type() {
        SampleType::U8 => {
            format::write(&hazard.evaluate::<u8>(grid, &values)?, description, output)
        }
        SampleType::F32 => {
            format::write(&hazard.evaluate::<f32>(grid, &values)?, description, output)
        }
    }
}

/// Value of each input of `hazard`, read from its text on the command line
/// according to the kind of the input.
fn read_values(hazard: &Hazard, texts: &[Option<&String>]) -> Result<Vec<Option<Value>>> {
    hazard
        .inputs()
        .iter()
        .zip(texts)
        .map(|(input, text)| {
            text.map(|text| {
                read_value(input.kind, text)
                    .with_context(|| format!("invalid value for input `{}`", input.name))
            })
            .transpose()
        })
        .collect()
}

/// Value of kind `kind` written as `text`: the path or URL of a raster file,
/// or a number.
fn read_value(kind: Kind, text: &str) -> Result<Value> {
    match kind {
        Kind::Raster => format::read_layer(&Location::parse(text)?).map(Value::Raster),
        Kind::Number => text
            .parse()
            .ok()
            .filter(|number: &f64| number.is_finite())
            .map(Value::Number)
            .with_context(|| format!("expected a finite number, found `{text}`")),
    }
}

/// Grid of `like`, or else of the first raster input of `hazard` that is
/// bound.
fn output_grid(
    like: Option<&Location>,
    hazard: &Hazard,
    texts: &[Option<&String>],
) -> Result<Grid> {
    if let Some(like) = like {
        return format::read_grid(like);
    }
    let first_raster = hazard
        .inputs()
        .iter()
        .zip(texts)
        .find_map(|(input, text)| text.filter(|_| input.kind == Kind::Raster))
        .context("no raster input to take the output grid from; pass --like")?;
    format::read_grid(&Location::parse(first_raster)?)
}

fn binding(text: &str) -> Result<(String, String), String> {
    text.split_once('=')
        .map(|(name, value)| (name.to_owned(), value.to_owned()))
        .ok_or_else(|| format!("expected NAME=VALUE, found `{text}`"))
}
