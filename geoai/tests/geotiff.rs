// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Rasters written to and read from GeoTIFF files.

use std::{fs::File, path::PathBuf};

use anyhow::Result;
use geoai::{
    format::{self, geotiff},
    geo::{Crs, Grid},
    raster::Raster,
    storage::Location,
};
use tiff::{
    encoder::{
        TiffEncoder, TiffValue,
        colortype::{ColorType, Gray8, GrayI16},
    },
    tags::Tag,
};

/// GeoKeyDirectory header declaring no keys.
const NO_KEYS: &[u16] = &[1, 1, 0, 0];

fn path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name)
}

fn grid() -> Grid {
    Grid::new(3, 2)
        .with_origin(10.0, 50.0)
        .with_pixel_size(0.5, 0.25)
        .with_crs(Crs::WGS84)
}

/// GeoTIFF named `name` of a row of `samples` of the color type `C`, on unit
/// pixels tied at (10.5, 49.5), with the GeoKeyDirectory `keys` and the GDAL
/// nodata text `nodata`.
fn tiff<C: ColorType>(
    name: &str,
    keys: &[u16],
    nodata: Option<&str>,
    samples: &[C::Inner],
) -> Result<Location>
where
    [C::Inner]: TiffValue,
{
    let path = path(name);
    let mut encoder = TiffEncoder::new(File::create(&path)?)?;
    let mut image = encoder.new_image::<C>(samples.len() as u32, 1)?;
    let tags = image.encoder();
    tags.write_tag(Tag::ModelPixelScaleTag, &[1.0, 1.0, 0.0][..])?;
    tags.write_tag(Tag::ModelTiepointTag, &[0.0, 0.0, 0.0, 10.5, 49.5, 0.0][..])?;
    tags.write_tag(Tag::GeoKeyDirectoryTag, keys)?;
    if let Some(nodata) = nodata {
        tags.write_tag(Tag::GdalNodata, nodata)?;
    }
    image.write_data(samples)?;
    Ok(Location::File(path))
}

#[test]
fn rasters_read_back_as_written() -> Result<()> {
    let file = Location::File(path("round-trip.tif"));
    let depth = Raster::new(grid(), vec![1.0, -9999.0, 3.5, 4.0, 5.0, 6.0])?.with_nodata(-9999.0);
    format::write(&depth, "depth in m³", &file)?;
    let read: Raster<f32> = format::read(&file)?;
    assert_eq!(read.grid(), depth.grid());
    assert_eq!(read.nodata(), depth.nodata());
    assert_eq!(read.data(), depth.data());

    let utm = grid().with_crs(Crs::Projected(32633));
    let classes = Raster::new(utm, vec![0_u8, 1, 2, 3, 4, 5])?.with_nodata(0);
    format::write(&classes, "classes", &file)?;
    let read: Raster<f32> = format::read(&file)?;
    assert_eq!(read.grid(), classes.grid());
    assert_eq!(read.data(), [0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
    assert_eq!(read.nodata(), Some(0.0));
    Ok(())
}

#[test]
fn files_are_read_by_content_and_written_by_extension() -> Result<()> {
    let path = path("raster.data");
    let raster = Raster::new(grid(), vec![0.0_f32; 6])?;
    geotiff::Writer::new().write(&raster, File::create(&path)?)?;
    let file = Location::File(path);
    assert_eq!(format::read_grid(&file)?, grid());
    assert!(format::write(&raster, "raster", &file).is_err());
    Ok(())
}

/// In a GeoTIFF whose pixels represent points, tie points locate pixel
/// centers rather than corners.
#[test]
fn points_are_tied_at_pixel_centers() -> Result<()> {
    // GTRasterTypeGeoKey declaring RasterPixelIsPoint
    let keys = [1, 1, 0, 1, 1025, 0, 1, 2];
    let file = tiff::<Gray8>("pixel-is-point.tif", &keys, None, &[0, 0])?;
    assert_eq!(format::read_grid(&file)?.origin(), (10.0, 50.0));
    Ok(())
}

/// A projected CRS that other keys define has no EPSG code, even when the
/// geographic CRS it is based on has one.
#[test]
fn crs_without_an_epsg_code_is_unknown() -> Result<()> {
    [
        // WGS 84 GeographicTypeGeoKey, and a user-defined ProjectedCSTypeGeoKey
        &[1, 1, 0, 2, 2048, 0, 1, 4326, 3072, 0, 1, 32767][..],
        // Projected GTModelTypeGeoKey, WGS 84, and a ProjCoordTransGeoKey
        &[1, 1, 0, 3, 1024, 0, 1, 1, 2048, 0, 1, 4326, 3075, 0, 1, 11][..],
    ]
    .into_iter()
    .try_for_each(|keys| {
        let file = tiff::<Gray8>("user-defined-crs.tif", keys, None, &[0, 0])?;
        assert_eq!(format::read_grid(&file)?.crs(), None, "keys {keys:?}");
        Ok(())
    })
}

/// The nodata value of integer samples may be written as a decimal, or as a
/// value that no sample can hold.
#[test]
fn nodata_values_are_read_as_numbers() -> Result<()> {
    let file = tiff::<Gray8>("nodata-decimal.tif", NO_KEYS, Some("255.0"), &[7, 255])?;
    let layer = format::read_layer(&file)?;
    assert_eq!(
        (layer.sample(11.0, 49.0), layer.sample(12.0, 49.0)),
        (Some(7.0), None)
    );
    let file = tiff::<Gray8>("nodata-out-of-range.tif", NO_KEYS, Some("-9999"), &[7, 255])?;
    let layer = format::read_layer(&file)?;
    assert_eq!(layer.sample(12.0, 49.0), Some(255.0));
    let file = tiff::<GrayI16>("nodata-signed.tif", NO_KEYS, Some("-9999"), &[-9999, -5])?;
    let layer = format::read_layer(&file)?;
    assert_eq!(
        (layer.sample(11.0, 49.0), layer.sample(12.0, 49.0)),
        (None, Some(-5.0))
    );
    Ok(())
}
