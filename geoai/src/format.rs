// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Raster file formats, recognized by their content when read and by their
//! extension when written.

pub mod geotiff;

use std::{
    io::{Cursor, Read},
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use tiff::encoder::{Compression, TiffValue};

use crate::{
    geo::{Grid, Layer},
    raster::Raster,
    storage::{Location, Source},
};

/// Read the raster at `location` with samples of type `T`, to which its own
/// samples must convert without loss.
pub fn read<T: geotiff::Sample>(location: &Location) -> Result<Raster<T>> {
    open(location)
        .and_then(|(format, source)| match format {
            Format::GeoTiff => geotiff::read(source),
        })
        .with_context(|| format!("cannot read {location}"))
}

/// Read the raster at `location` as a layer of whichever sample type it holds.
pub fn read_layer(location: &Location) -> Result<Arc<dyn Layer>> {
    open(location)
        .and_then(|(format, source)| match format {
            Format::GeoTiff => geotiff::read_layer(source),
        })
        .with_context(|| format!("cannot read {location}"))
}

/// Read the grid of the raster at `location` without decoding its pixels.
pub fn read_grid(location: &Location) -> Result<Grid> {
    open(location)
        .and_then(|(format, source)| match format {
            Format::GeoTiff => geotiff::read_grid(source),
        })
        .with_context(|| format!("cannot read {location}"))
}

/// Ensure that a raster can be written to `location`: that its extension
/// names a format and, for a local file, that its directory exists.  Checking
/// first saves computing a raster that cannot be written.
pub fn check_writable(location: &Location) -> Result<()> {
    Format::of_extension(location)
        .and_then(|_| location.check_writable())
        .with_context(|| format!("cannot write {location}"))
}

/// Write `raster` to `location`, with `description` as its band description.
pub fn write<T: geotiff::Sample>(
    raster: &Raster<T>,
    description: &str,
    location: &Location,
) -> Result<()>
where
    [T]: TiffValue,
{
    Format::of_extension(location)
        .and_then(|format| {
            let mut file = Cursor::new(Vec::new());
            match format {
                Format::GeoTiff => geotiff::Writer::new()
                    .compression(Compression::Lzw)
                    .description(description)
                    .write(raster, &mut file)?,
            }
            location.write(file.into_inner())
        })
        .with_context(|| format!("cannot write {location}"))
}

/// Format of the file at `location`, and the file open at its start.
fn open(location: &Location) -> Result<(Format, Box<dyn Source>)> {
    let mut source = location.open()?;
    Ok((Format::of_content(&mut source)?, source))
}

enum Format {
    GeoTiff,
}

impl Format {
    /// Format of `source`, recognized by its signature, leaving `source` at
    /// its start.
    fn of_content(source: &mut impl Source) -> Result<Self> {
        let mut signature = Vec::new();
        source.by_ref().take(4).read_to_end(&mut signature)?;
        source.rewind()?;
        match signature.as_slice() {
            // TIFF and BigTIFF, in little- and big-endian byte order
            b"II*\0" | b"MM\0*" | b"II+\0" | b"MM\0+" => Ok(Self::GeoTiff),
            _ => bail!("unsupported file format"),
        }
    }

    /// Format that the extension of `location` names.
    fn of_extension(location: &Location) -> Result<Self> {
        let extension = location.extension().unwrap_or_default();
        match extension.to_ascii_lowercase().as_str() {
            "tif" | "tiff" => Ok(Self::GeoTiff),
            _ => bail!("expected a .tif or .tiff file extension"),
        }
    }
}
