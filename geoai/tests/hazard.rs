// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Hazards defined in Scheme: signatures, declarations, and bodies.

mod common;

use std::sync::Arc;

use anyhow::Result;
use common::{hazard, row};
use geoai::{
    geo::{Crs, Grid},
    hazard::{SampleType, Value},
    raster::Raster,
};
use itertools::Itertools;

/// Values of the hazard defined by `source` along a row of `width` pixels.
fn values(source: &str, width: u32, inputs: &[Option<Value>]) -> Result<Vec<f32>> {
    Ok(hazard(source)?
        .evaluate::<f32>(Grid::new(width, 1), inputs)?
        .data()
        .to_vec())
}

/// Message of the error that computing the hazard defined by `source` along a
/// single pixel causes.
fn error(source: &str, inputs: &[Option<Value>]) -> String {
    format!("{:#}", values(source, 1, inputs).unwrap_err())
}

#[test]
fn signatures_declare_inputs_and_output() -> Result<()> {
    let hazard = hazard(
        "(define-hazard (h pga (optional land) (threshold number) (optional factor number))
           (type u8)
           (description \"class\")
           (nodata 255)
           (pixelwise () 0))",
    )?;
    assert_eq!(
        hazard.inputs().iter().join(" "),
        "pga [land] threshold:number [factor:number]"
    );
    assert_eq!(hazard.sample_type(), SampleType::U8);
    assert_eq!(hazard.description(), "class");
    Ok(())
}

#[test]
fn number_inputs_reach_the_body() -> Result<()> {
    let source = "
        (define-hazard (flood depth (threshold number))
          (type u8)
          (nodata 255)
          (pixelwise (depth threshold)
            (cond ((> depth ,(* 2 threshold)) 2)
                  ((> depth threshold) 1)
                  (else 0))))";
    let inputs = [Some(row(&[0.2, 0.8, 1.2])?), Some(Value::Number(0.5))];
    assert_eq!(values(source, 3, &inputs)?, [0.0, 1.0, 2.0]);
    Ok(())
}

#[test]
fn unbound_optional_inputs_are_false() -> Result<()> {
    let source = "
        (define-hazard (h (optional factor number))
          (pixelwise () ,(if factor factor 1)))";
    assert_eq!(values(source, 1, &[None])?, [1.0]);
    assert_eq!(values(source, 1, &[Some(Value::Number(3.0))])?, [3.0]);
    Ok(())
}

#[test]
fn missing_values_become_nodata() -> Result<()> {
    let source = "(define-hazard (h a) (nodata -1) (pixelwise (a) (* a 2)))";
    assert_eq!(
        values(source, 2, &[Some(row(&[1.0, f32::NAN])?)])?,
        [2.0, -1.0]
    );
    // Lowest f32 as Rust and NumPy print it, a common nodata value
    let source = "(define-hazard (h a) (nodata -3.4028235e38) (pixelwise (a) a))";
    assert_eq!(values(source, 1, &[Some(row(&[f32::NAN])?)])?, [f32::MIN]);
    Ok(())
}

#[test]
fn rasters_are_missing_outside_their_grid() -> Result<()> {
    let source = "(define-hazard (h a) (nodata -1) (pixelwise (a) a))";
    assert_eq!(
        values(source, 4, &[Some(row(&[1.0, 2.0])?)])?,
        [1.0, 2.0, -1.0, -1.0]
    );
    Ok(())
}

#[test]
fn missing_values_without_nodata_are_errors() -> Result<()> {
    let source = "(define-hazard (h a) (pixelwise (a) (* a 2)))";
    let error = error(source, &[Some(row(&[f32::NAN])?)]);
    assert!(error.contains("no nodata"), "{error}");
    Ok(())
}

#[test]
fn values_outside_the_output_type_are_errors() -> Result<()> {
    let error = hazard("(define-hazard (h) (type u8) (pixelwise () 300))")?
        .evaluate::<u8>(Grid::new(1, 1), &[])
        .unwrap_err();
    assert!(
        format!("{error:#}").contains("300 does not fit in u8"),
        "{error:#}"
    );
    Ok(())
}

#[test]
fn bodies_must_return_rasters() {
    let error = error("(define-hazard (h) 42)", &[]);
    assert!(
        error.contains("expected the body to return a raster, found 42"),
        "{error}"
    );
}

#[test]
fn errors_in_bodies_quote_their_source() {
    let error = error("(define-hazard (h)\n  (car '()))", &[]);
    assert!(
        error.contains("test.scm:2") && error.contains("car"),
        "{error}"
    );
}

#[test]
fn values_must_be_of_the_kind_of_their_input() -> Result<()> {
    let source = "(define-hazard (h (threshold number)) (pixelwise () 1))";
    let error = error(source, &[Some(row(&[1.0])?)]);
    assert!(
        error.contains("input `threshold` takes a number, not a raster"),
        "{error}"
    );
    Ok(())
}

#[test]
fn rasters_must_be_in_the_crs_of_the_output() -> Result<()> {
    let utm = Raster::new(
        Grid::new(1, 1).with_crs(Crs::Projected(32633)),
        vec![1.0_f32],
    )?;
    let error = hazard("(define-hazard (h a) (pixelwise (a) a))")?
        .evaluate::<f32>(
            Grid::new(1, 1).with_crs(Crs::WGS84),
            &[Some(Value::Raster(Arc::new(utm)))],
        )
        .unwrap_err();
    assert!(
        format!("{error:#}").contains("input `a` is in EPSG:32633"),
        "{error:#}"
    );
    Ok(())
}

#[test]
fn invalid_definitions_are_rejected() {
    [
        (
            "(define-hazard (h a a) (pixelwise (a) a))",
            "declared more than once",
        ),
        (
            "(define-hazard (h a) (type u16) (pixelwise (a) a))",
            "unknown sample type `u16`",
        ),
        (
            "(define-hazard (h (a vector)) (pixelwise () 1))",
            "unknown input kind `vector`",
        ),
        (
            "(define-hazard (h) (type u8) (type f32) (pixelwise () 1))",
            "`type` is declared more than once",
        ),
        (
            "(define-hazard (h) (type u8) (nodata 2.5) (pixelwise () 1))",
            "nodata 2.5 does not fit in u8",
        ),
        ("(define-hazard (h a) (type u8))", "hazard has no body: h"),
        (
            "(define-hazard (h) (pixelwise () 1)) (define-hazard (h) (pixelwise () 2))",
            "hazard `h` is defined more than once",
        ),
        ("(car '())", "car"),
    ]
    .into_iter()
    .for_each(|(source, message)| {
        let error = format!("{:#}", hazard(source).unwrap_err());
        assert!(error.contains(message), "{source}: {error}");
    });
}

#[test]
fn bind_orders_values_by_input() -> Result<()> {
    let hazard = hazard("(define-hazard (h a (optional b) c) (pixelwise (a) a))")?;
    let bindings = |names: &[&'static str]| {
        names
            .iter()
            .map(|name| (name.to_string(), *name))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        hazard.bind(&bindings(&["c", "a"]))?,
        [Some(&"a"), None, Some(&"c")]
    );
    [
        (bindings(&["c"]), "requires input `a`"),
        (bindings(&["a", "c", "d"]), "has no input `d`"),
        (bindings(&["a", "c", "a"]), "bound more than once"),
    ]
    .into_iter()
    .for_each(|(bindings, message)| {
        let error = hazard.bind(&bindings).unwrap_err().to_string();
        assert!(error.contains(message), "{error}");
    });
    Ok(())
}
