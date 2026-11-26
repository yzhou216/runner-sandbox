// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Rasters computed pixel by pixel from other rasters.
//!
//! A [`Pixelwise`] layer evaluates its expression wherever it is sampled,
//! from the values its input layers have at the same point.  It stores no
//! pixels, so pixelwise layers built on one another are still computed in a
//! single pass.

mod expression;
mod parse;

use std::sync::Arc;

use anyhow::{Result, bail};
use itertools::Itertools;
use smallvec::SmallVec;

use crate::{geo::Layer, sexp::Sexp};
use expression::Expression;
use parse::Parser;

/// Layer computed at each point by an expression over other layers.
pub struct Pixelwise {
    expression: Expression,
    /// Layer of each bound name, in order of binding, or `None` for a name
    /// whose value is missing everywhere.
    inputs: Vec<Option<Arc<dyn Layer>>>,
}

impl Pixelwise {
    /// Parse `expression`, in which the name of each binding stands for the
    /// value of its layer.
    pub fn new(expression: &Sexp, bindings: Vec<(String, Option<Arc<dyn Layer>>)>) -> Result<Self> {
        let names: Vec<&str> = bindings.iter().map(|(name, _)| name.as_str()).collect();
        if let Some(name) = names.iter().duplicates().next() {
            bail!("`{name}` is bound more than once");
        }
        let expression = Parser::new(&names).expression(expression)?;
        Ok(Self {
            expression,
            inputs: bindings.into_iter().map(|(_, layer)| layer).collect(),
        })
    }
}

/// Each input is sampled once, however often the expression uses it, so that
/// a pixelwise layer built on others costs no more than their sum.
impl Layer for Pixelwise {
    fn sample(&self, x: f64, y: f64) -> Option<f64> {
        let values: SmallVec<[_; 8]> = self
            .inputs
            .iter()
            .map(|layer| layer.as_ref()?.sample(x, y))
            .collect();
        self.expression.evaluate(&|input| values[input])
    }
}
