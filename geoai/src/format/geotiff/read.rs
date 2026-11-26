// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    any,
    io::{Read, Seek},
    str::FromStr,
    sync::Arc,
};

use anyhow::{Context, Result, bail, ensure};
use num_traits::FromPrimitive;
use tiff::{
    ColorType,
    decoder::{Decoder, DecodingResult, Limits},
    tags::Tag,
};

use super::{Sample, geokeys};
use crate::{
    geo::{Grid, Layer},
    raster::Raster,
};

/// Decode the first image of the GeoTIFF in `source`, ignoring overviews.
///
/// The image must have a single band whose samples convert to `T` without
/// loss.
pub fn read<T: Sample>(source: impl Read + Seek) -> Result<Raster<T>> {
    let (grid, nodata, image) = decode(source)?;
    let data = T::from_decoded(image).with_context(|| {
        format!(
            "samples do not convert to {} without loss",
            any::type_name::<T>()
        )
    })?;
    raster(grid, nodata.as_deref(), data)
}

/// Decode the first image of the GeoTIFF in `source` as a layer of whichever
/// sample type it holds: an integer of up to 32 bits, or a floating-point
/// number of 32 or 64 bits.
pub fn read_layer(source: impl Read + Seek) -> Result<Arc<dyn Layer>> {
    let (grid, nodata, image) = decode(source)?;
    let nodata = nodata.as_deref();
    Ok(match image {
        DecodingResult::U8(data) => Arc::new(raster(grid, nodata, data)?),
        DecodingResult::I8(data) => Arc::new(raster(grid, nodata, data)?),
        DecodingResult::U16(data) => Arc::new(raster(grid, nodata, data)?),
        DecodingResult::I16(data) => Arc::new(raster(grid, nodata, data)?),
        DecodingResult::U32(data) => Arc::new(raster(grid, nodata, data)?),
        DecodingResult::I32(data) => Arc::new(raster(grid, nodata, data)?),
        DecodingResult::F32(data) => Arc::new(raster(grid, nodata, data)?),
        DecodingResult::F64(data) => Arc::new(raster(grid, nodata, data)?),
        _ => bail!("64-bit integer and 16-bit floating-point samples are not supported"),
    })
}

/// Grid of the GeoTIFF in `source`, read without decoding any pixels.
pub fn read_grid(source: impl Read + Seek) -> Result<Grid> {
    grid(&mut Decoder::new(source)?)
}

/// Grid, nodata text, and pixels of the first image of the GeoTIFF in
/// `source`.
fn decode(source: impl Read + Seek) -> Result<(Grid, Option<String>, DecodingResult)> {
    let mut decoder = Decoder::new(source)?;
    let grid = grid(&mut decoder)?;
    let nodata = nodata(&mut decoder)?;
    let bits = match decoder.colortype()? {
        ColorType::Gray(bits) => usize::from(bits),
        other => bail!("expected a single grayscale band, found {other:?}"),
    };
    // Smaller samples arrive packed several to a byte
    ensure!(bits % 8 == 0, "{bits}-bit samples are not supported");
    let bytes = grid
        .len()
        .checked_mul(bits / 8)
        .context("image is too large")?;
    let image = decoder.with_limits(limits_for(bytes)).read_image()?;
    Ok((grid, nodata, image))
}

fn grid(decoder: &mut Decoder<impl Read + Seek>) -> Result<Grid> {
    let (width, height) = decoder.dimensions()?;
    let (&[pixel_width, pixel_height, ..], &[column, row, _, x, y, ..]) = (
        decoder.get_tag_f64_vec(Tag::ModelPixelScaleTag)?.as_slice(),
        decoder.get_tag_f64_vec(Tag::ModelTiepointTag)?.as_slice(),
    ) else {
        bail!("malformed ModelPixelScaleTag or ModelTiepointTag");
    };
    let positive = |size: f64| size > 0.0 && size.is_finite();
    ensure!(
        positive(pixel_width) && positive(pixel_height),
        "expected a positive pixel size, found {pixel_width} x {pixel_height}"
    );
    ensure!(
        [column, row, x, y].iter().all(|value| value.is_finite()),
        "expected a finite tie point, found ({column}, {row}) at ({x}, {y})"
    );
    let keys = decoder
        .find_tag_unsigned_vec::<u16>(Tag::GeoKeyDirectoryTag)?
        .unwrap_or_default();
    let offset = if geokeys::pixel_is_point(&keys) {
        0.5
    } else {
        0.0
    };
    Ok(Grid::new(width, height)
        .with_origin(
            x - (column + offset) * pixel_width,
            y + (row + offset) * pixel_height,
        )
        .with_pixel_size(pixel_width, pixel_height)
        .with_crs(geokeys::decode(&keys)))
}

/// GDAL nodata value as text, without the padding GDAL may append.
fn nodata(decoder: &mut Decoder<impl Read + Seek>) -> Result<Option<String>> {
    let Some(value) = decoder.find_tag(Tag::GdalNodata)? else {
        return Ok(None);
    };
    let text = value.into_string()?;
    Ok(Some(text.trim_end_matches('\0').trim().to_owned()))
}

fn raster<T>(grid: Grid, nodata: Option<&str>, data: Vec<T>) -> Result<Raster<T>>
where
    T: Copy + PartialEq + FromStr + FromPrimitive + Into<f64>,
{
    let nodata = nodata.map(nodata_value).transpose()?.flatten();
    Ok(Raster::new(grid, data)?.with_nodata(nodata))
}

/// Nodata value written as `text`, which may not parse as `T`, as with 255.0
/// for u8 samples.  A value that no sample can equal, such as -9999 for u8,
/// marks no sample as nodata.
fn nodata_value<T>(text: &str) -> Result<Option<T>>
where
    T: Copy + FromStr + FromPrimitive + Into<f64>,
{
    if let Ok(value) = text.parse() {
        return Ok(Some(value));
    }
    let value: f64 = text
        .parse()
        .ok()
        .with_context(|| format!("invalid GDAL nodata value {text:?}"))?;
    Ok(T::from_f64(value).filter(|&sample| sample.into() == value))
}

/// Default limits with the caps on the decoded image, 256 MiB, and on each
/// strip or tile, 128 MiB, raised to `bytes`.
fn limits_for(bytes: usize) -> Limits {
    let mut limits = Limits::default();
    limits.decoding_buffer_size = limits.decoding_buffer_size.max(bytes);
    limits.intermediate_buffer_size = limits.intermediate_buffer_size.max(bytes);
    limits
}
