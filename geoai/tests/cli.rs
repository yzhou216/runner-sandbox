// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The geoai command, run as a user would run it.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use anyhow::{Result, ensure};
use geoai::{format, geo::Grid, raster::Raster, storage::Location};

const FLOOD: &str = "
    (define-hazard (flood depth (threshold number))
      (type u8)
      (nodata 255)
      (pixelwise (depth threshold)
        (cond ((> depth ,(* 2 threshold)) 2)
              ((> depth threshold) 1)
              (else 0))))";

/// Empty directory for the test named `name`, apart from the flood hazard in
/// `flood.scm` and a row of depths in `depth.tif`.  It also serves as
/// `$XDG_CONFIG_HOME`, so that the hazards of the user stay out of the test.
fn directory(name: &str) -> Result<PathBuf> {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("cli")
        .join(name);
    if directory.exists() {
        fs::remove_dir_all(&directory)?;
    }
    fs::create_dir_all(&directory)?;
    fs::write(directory.join("flood.scm"), FLOOD)?;
    let depth = Raster::new(Grid::new(3, 1), vec![0.2_f32, 0.8, 1.2])?;
    format::write(&depth, "depth", &file(&directory, "depth.tif"))?;
    Ok(directory)
}

fn file(directory: &Path, name: &str) -> Location {
    Location::File(directory.join(name))
}

/// Output of geoai, run in `directory` with `arguments`.
fn run(directory: &Path, arguments: &[&str]) -> Result<Output> {
    Ok(Command::new(env!("CARGO_BIN_EXE_geoai"))
        .current_dir(directory)
        .env("XDG_CONFIG_HOME", directory)
        .args(arguments)
        .output()?)
}

/// Standard output of geoai, run in `directory` with `arguments`, which must
/// succeed.
fn geoai(directory: &Path, arguments: &[&str]) -> Result<String> {
    let output = run(directory, arguments)?;
    ensure!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?)
}

/// Standard error of geoai, run in `directory` with `arguments`, which must
/// fail.
fn failure(directory: &Path, arguments: &[&str]) -> Result<String> {
    let output = run(directory, arguments)?;
    ensure!(!output.status.success(), "geoai succeeded");
    Ok(String::from_utf8(output.stderr)?)
}

/// Arguments that compute the flood hazard into `flood.tif`, with `depth` as
/// the argument for its depth input, followed by `extra`.
fn flood<'a>(depth: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let run = ["run", "flood", "flood.tif", "--hazards", "flood.scm"];
    let inputs = ["--input", depth, "--input", "threshold=0.5"];
    run.into_iter()
        .chain(inputs)
        .chain(extra.iter().copied())
        .collect()
}

#[test]
fn run_writes_a_hazard_that_info_describes() -> Result<()> {
    let directory = directory("run")?;
    geoai(&directory, &flood("depth=depth.tif", &[]))?;
    let flood: Raster<u8> = format::read(&file(&directory, "flood.tif"))?;
    assert_eq!(flood.data(), [0, 1, 2]);
    let info = geoai(&directory, &["info", "flood.tif"])?;
    assert!(info.contains("count=3"), "{info}");
    Ok(())
}

#[test]
fn like_sets_the_output_grid() -> Result<()> {
    let directory = directory("like")?;
    let like = Grid::new(6, 2).with_pixel_size(0.5, 0.5);
    format::write(
        &Raster::new(like, vec![0_u8; 12])?,
        "like",
        &file(&directory, "like.tif"),
    )?;
    geoai(
        &directory,
        &flood("depth=depth.tif", &["--like", "like.tif"]),
    )?;
    assert_eq!(format::read_grid(&file(&directory, "flood.tif"))?, like);
    Ok(())
}

#[test]
fn failures_report_their_cause() -> Result<()> {
    let directory = directory("failure")?;
    let error = failure(&directory, &flood("depth=missing.tif", &[]))?;
    assert!(
        error.starts_with("error: ") && error.contains("cannot read missing.tif: "),
        "{error}"
    );
    Ok(())
}

#[test]
fn list_shows_the_inputs_of_each_hazard() -> Result<()> {
    let directory = directory("list")?;
    let list = geoai(&directory, &["list", "--hazards", "flood.scm"])?;
    assert!(list.contains("depth threshold:number"), "{list}");
    Ok(())
}

/// The user directory replaces the built-in earthquake hazard, and a path
/// given with `--hazards` replaces the flood hazard of the user directory.
#[test]
fn later_hazards_replace_earlier_ones() -> Result<()> {
    let directory = directory("order")?;
    let user = directory.join("geoai").join("hazards");
    fs::create_dir_all(&user)?;
    fs::write(
        user.join("earthquake.scm"),
        "(define-hazard (earthquake pga) (pixelwise (pga) pga))",
    )?;
    fs::write(
        user.join("flood.scm"),
        "(define-hazard (flood depth) (pixelwise (depth) depth))",
    )?;
    let list = geoai(&directory, &["list", "--hazards", "flood.scm"])?;
    let line = |name: &str| {
        list.lines()
            .find(|line| line.starts_with(name))
            .unwrap_or_default()
    };
    assert!(
        line("earthquake ").ends_with("geoai/hazards/earthquake.scm"),
        "{list}"
    );
    assert!(line("flood ").ends_with(" flood.scm"), "{list}");
    Ok(())
}
