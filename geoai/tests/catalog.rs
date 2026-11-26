// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Hazards loaded from directories of Scheme files.

use std::{fs, path::PathBuf, sync::Arc};

use anyhow::{Context, Result, bail};
use geoai::{
    catalog::{Catalog, Source},
    geo::Grid,
    hazard::Value,
    raster::Raster,
};

/// Empty directory for the test named `name`.
fn directory(name: &str) -> Result<PathBuf> {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    if directory.exists() {
        fs::remove_dir_all(&directory)?;
    }
    fs::create_dir_all(&directory)?;
    Ok(directory)
}

/// Value of the hazard `name` of `catalog` at a single pixel, where its only
/// input is `depth`.
fn compute(catalog: &Catalog, name: &str, depth: f32) -> Result<f32> {
    let grid = Grid::new(1, 1);
    let depth = Value::Raster(Arc::new(Raster::new(grid, vec![depth])?));
    let hazard = catalog
        .get(name)
        .with_context(|| format!("no hazard `{name}`"))?;
    Ok(hazard.evaluate::<f32>(grid, &[Some(depth)])?.data()[0])
}

#[test]
fn directory_hazards_replace_builtin_ones() -> Result<()> {
    let directory = directory("catalog")?;
    fs::write(
        directory.join("earthquake.scm"),
        "(define-hazard (earthquake pga) (pixelwise () 7))",
    )?;
    fs::write(
        directory.join("flood.scm"),
        "(define-hazard (flood depth) (pixelwise (depth) depth))",
    )?;
    fs::write(directory.join("notes.txt"), "Not Scheme")?;
    fs::write(directory.join(".#flood.scm"), "Lock file of an editor")?;

    let catalog = Catalog::builtin()?.with_path(&directory)?;
    let names: Vec<_> = catalog.entries().map(|entry| entry.hazard.name()).collect();
    assert_eq!(names, ["earthquake", "flood"]);

    let earthquake = catalog
        .entries()
        .find(|entry| entry.hazard.name() == "earthquake")
        .context("no earthquake hazard")?;
    assert_eq!(
        earthquake.source,
        Source::File(directory.join("earthquake.scm"))
    );
    assert_eq!(earthquake.hazard.inputs().len(), 1);
    Ok(())
}

#[test]
fn files_may_require_the_same_library() -> Result<()> {
    let directory = directory("library")?;
    fs::create_dir(directory.join("library"))?;
    fs::write(
        directory.join("library/common.scm"),
        "(provide doubled offset fail)
         (define offset 10)
         (define (doubled raster) (pixelwise (raster) (* raster 2)))
         (define (fail) (car '()))",
    )?;
    fs::write(
        directory.join("a.scm"),
        "(require \"library/common.scm\")
         (define-hazard (a depth) (doubled depth))",
    )?;
    fs::write(
        directory.join("b.scm"),
        "(require \"library/common.scm\")
         (define-hazard (b depth) (pixelwise (depth) (+ depth ,offset)))
         (define-hazard (c depth) (fail))",
    )?;

    let catalog = Catalog::builtin()?.with_path(&directory)?;
    assert_eq!(compute(&catalog, "a", 1.0)?, 2.0);
    assert_eq!(compute(&catalog, "b", 1.0)?, 11.0);
    let error = format!("{:#}", compute(&catalog, "c", 1.0).unwrap_err());
    assert!(error.contains("library/common.scm"), "{error}");
    Ok(())
}

#[test]
fn files_share_no_definitions() -> Result<()> {
    let directory = directory("isolation")?;
    fs::write(directory.join("a.scm"), "(define offset 10)")?;
    fs::write(
        directory.join("b.scm"),
        "(define-hazard (b) (pixelwise () ,offset))",
    )?;
    let Err(error) = Catalog::builtin()?.with_path(&directory) else {
        bail!("b.scm loaded with a definition from a.scm");
    };
    assert!(format!("{error:#}").contains("offset"), "{error:#}");
    Ok(())
}
