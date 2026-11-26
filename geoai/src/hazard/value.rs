// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use super::Kind;
use crate::geo::Layer;

/// Value bound to an input of a hazard.
#[derive(Clone)]
pub enum Value {
    Raster(Arc<dyn Layer>),
    Number(f64),
}

impl Value {
    pub fn kind(&self) -> Kind {
        match self {
            Self::Raster(_) => Kind::Raster,
            Self::Number(_) => Kind::Number,
        }
    }
}
