// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Rasters kept in object stores, and the locations that name files.

use std::{path::Path, sync::Arc};

use anyhow::Result;
use geoai::{format, geo::Grid, raster::Raster, storage::Location};
use object_store::memory::InMemory;
use url::Url;

/// A raster written to an object store reads back, as it does from S3.
#[test]
fn rasters_read_back_from_an_object_store() -> Result<()> {
    let location = Location::Object {
        url: Url::parse("memory:///runs/flood.tif")?,
        store: Arc::new(InMemory::new()),
        path: "runs/flood.tif".into(),
    };
    let flood = Raster::new(Grid::new(3, 1), vec![0_u8, 1, 2])?.with_nodata(255);
    format::write(&flood, "flood", &location)?;
    let read: Raster<u8> = format::read(&location)?;
    assert_eq!(read.grid(), flood.grid());
    assert_eq!(read.data(), flood.data());
    Ok(())
}

/// Paths and `file://` URLs name local files, and a URL of no known store is
/// an error.
#[test]
fn paths_and_file_urls_name_local_files() -> Result<()> {
    assert!(matches!(
        Location::parse("runs/flood.tif")?,
        Location::File(path) if path == Path::new("runs/flood.tif")
    ));
    assert!(matches!(
        Location::parse("file:///data/flood.tif")?,
        Location::File(path) if path == Path::new("/data/flood.tif")
    ));
    assert!(Location::parse("ftp://host/flood.tif").is_err());
    Ok(())
}
