// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use anyhow::Result;

use super::Value;
use crate::geo::Layer;

/// Computation of a hazard from the values bound to its inputs.
///
/// Each language in which hazards can be written implements this trait, as
/// [`crate::script`] does for Scheme.  The layer a body returns is then
/// sampled at every pixel of the output.
pub trait Body {
    /// Layer of hazard values, given one value per input in order and `None`
    /// for an unbound optional input.
    fn compute(&self, values: &[Option<Value>]) -> Result<Arc<dyn Layer>>;
}
