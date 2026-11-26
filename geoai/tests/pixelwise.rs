// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Expressions computed at every pixel by `pixelwise`.

mod common;

use anyhow::Result;
use common::{hazard, row};
use geoai::geo::Grid;

/// Values of the hazard defined by `source` along a row of pixels, given the
/// values of each input there, with NaN marking a missing value.
fn values(source: &str, inputs: &[&[f32]]) -> Result<Vec<f32>> {
    let width = inputs.first().map_or(1, |values| values.len());
    let inputs = inputs
        .iter()
        .map(|values| row(values).map(Some))
        .collect::<Result<Vec<_>>>()?;
    Ok(hazard(source)?
        .evaluate::<f32>(Grid::new(width as u32, 1), &inputs)?
        .data()
        .to_vec())
}

#[test]
fn arithmetic_and_comparisons() -> Result<()> {
    let source = "
        (define-hazard (h a b)
          (pixelwise (a b) (if (< a b) (+ a b 1) (max a b))))";
    assert_eq!(values(source, &[&[1.0, 5.0], &[2.0, 3.0]])?, [4.0, 5.0]);
    Ok(())
}

#[test]
fn operators_apply_from_left_to_right() -> Result<()> {
    let source = "
        (define-hazard (h a)
          (pixelwise (a)
            (cond ((or (= a 1) (>= a 9)) (- 10 a 2))
                  ((<= a 4) (/ 64 a 2))
                  (else (min a 6 7)))))";
    assert_eq!(
        values(source, &[&[1.0, 9.0, 4.0, 8.0]])?,
        [7.0, -1.0, 8.0, 6.0]
    );
    Ok(())
}

/// A table of clauses as long as one a program might generate.
#[test]
fn long_conds_are_evaluated() -> Result<()> {
    let source = "
        (define clauses
          (map (lambda (value) (list (list '= 'a value) value)) (range 0 50000)))
        (define-hazard (h a) (pixelwise (a) (cond ,@clauses (else -1))))";
    assert_eq!(values(source, &[&[49999.0, 0.5]])?, [49999.0, -1.0]);
    Ok(())
}

#[test]
fn cond_takes_the_first_clause_that_holds() -> Result<()> {
    let source = "
        (define-hazard (h a)
          (pixelwise (a) (cond ((< a 1) 10) ((< a 2) 20) (else 30))))";
    assert_eq!(values(source, &[&[0.5, 1.5, 2.5]])?, [10.0, 20.0, 30.0]);
    Ok(())
}

#[test]
fn missing_operands_make_the_result_missing() -> Result<()> {
    let source = "(define-hazard (h a b) (nodata -1) (pixelwise (a b) (+ a b)))";
    assert_eq!(
        values(source, &[&[1.0, f32::NAN], &[2.0, 3.0]])?,
        [3.0, -1.0]
    );
    Ok(())
}

#[test]
fn results_that_are_not_finite_are_missing() -> Result<()> {
    let source = "(define-hazard (h a) (nodata -1) (pixelwise (a) (/ 1 a)))";
    assert_eq!(values(source, &[&[0.0, 2.0]])?, [-1.0, 0.5]);
    Ok(())
}

#[test]
fn and_stops_at_a_false_guard() -> Result<()> {
    let source = "
        (define-hazard (h a)
          (pixelwise (a) (if (and (not (missing? a)) (> a 0)) 1 0)))";
    assert_eq!(values(source, &[&[f32::NAN, 1.0, -1.0]])?, [0.0, 1.0, 0.0]);
    Ok(())
}

#[test]
fn names_may_stand_for_numbers_and_missing_values() -> Result<()> {
    let source = "
        (define-hazard (h a)
          (define offset 10)
          (define nothing #f)
          (pixelwise (a offset nothing)
            (if (missing? nothing) (+ a offset) 0)))";
    assert_eq!(values(source, &[&[1.0, 2.0]])?, [11.0, 12.0]);
    Ok(())
}

#[test]
fn unquote_inserts_values_computed_by_the_body() -> Result<()> {
    let source = "
        (define classes '(1 2))
        (define-hazard (h a)
          (define threshold (* 5 2))
          (pixelwise (a) (if (< a ,threshold) ,@classes)))";
    assert_eq!(values(source, &[&[5.0, 15.0]])?, [1.0, 2.0]);
    Ok(())
}

#[test]
fn exact_fractions_are_numbers() -> Result<()> {
    let source = "
        (define-hazard (h a)
          (define half (/ 1 2))
          (pixelwise (a half) (+ (* a half) ,(/ 1 4))))";
    assert_eq!(values(source, &[&[2.0]])?, [1.25]);
    Ok(())
}

#[test]
fn pixelwise_rasters_feed_one_another() -> Result<()> {
    let source = "
        (define-hazard (h a)
          (define doubled (pixelwise (a) (* a 2)))
          (pixelwise (a doubled) (+ a doubled)))";
    assert_eq!(values(source, &[&[1.0, 2.0]])?, [3.0, 6.0]);
    Ok(())
}

#[test]
fn invalid_expressions_are_rejected() {
    [
        ("(pixelwise (a) b)", "unbound name `b`"),
        ("(pixelwise (a) (log a))", "unknown operator `log`"),
        ("(pixelwise (a) (if (+ a 1) 1 0))", "unknown condition `+`"),
        ("(pixelwise (a) (- a))", "at least two operands"),
        ("(pixelwise (a) (cond ((< a 1) 2)))", "else"),
        ("(pixelwise (a a) a)", "bound more than once"),
        ("(pixelwise ((+ 1 2)) 1)", "expected a name to bind"),
        ("(pixelwise (\"a\") 1)", "expected a name to bind"),
        (
            "(let ((label \"text\")) (pixelwise (label) 1))",
            "expected a raster, a number, or #f",
        ),
        (
            "(pixelwise (a) (if (< a +inf.0) 1 0))",
            "expected a finite number",
        ),
        (
            "(let ((cap (/ 1. 0.))) (pixelwise (a cap) (if (< a cap) 1 0)))",
            "expected a finite number",
        ),
    ]
    .into_iter()
    .for_each(|(body, message)| {
        let source = format!("(define-hazard (h a) {body})");
        let error = format!("{:#}", values(&source, &[&[1.0]]).unwrap_err());
        assert!(error.contains(message), "{body}: {error}");
    });
}
