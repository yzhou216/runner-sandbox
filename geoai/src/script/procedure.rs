// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{cell::RefCell, rc::Rc, sync::Arc};

use anyhow::{Context, Result};
use steel::{SteelVal, steel_vm::engine::Engine};

use super::{primitives, render};
use crate::{
    geo::Layer,
    hazard::{Body, Value},
};

/// Engine in which a hazard file ran, with the name of the file for errors.
pub struct Environment {
    engine: RefCell<Engine>,
    origin: String,
}

impl Environment {
    pub fn new(engine: Engine, origin: &str) -> Self {
        Self {
            engine: RefCell::new(engine),
            origin: origin.to_owned(),
        }
    }
}

/// Body of a hazard written as a Scheme procedure, called in the environment
/// of the file that defined it.
pub struct Procedure {
    procedure: SteelVal,
    environment: Rc<Environment>,
}

impl Procedure {
    pub fn new(procedure: SteelVal, environment: Rc<Environment>) -> Self {
        Self {
            procedure,
            environment,
        }
    }
}

impl Body for Procedure {
    fn compute(&self, values: &[Option<Value>]) -> Result<Arc<dyn Layer>> {
        let Environment { engine, origin } = self.environment.as_ref();
        let arguments = values
            .iter()
            .map(|value| primitives::to_scheme(value.as_ref()))
            .collect::<Result<_>>()?;
        let mut engine = engine.borrow_mut();
        let result = engine
            .call_function_with_args(self.procedure.clone(), arguments)
            .map_err(|error| render(&engine, origin, &error))?;
        primitives::layer(&result)
            .with_context(|| format!("expected the body to return a raster, found {result}"))
    }
}
