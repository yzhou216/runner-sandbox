// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Hazards written in Scheme and run with Steel.
//!
//! A hazard file is an ordinary Steel program, run in an engine of its own so
//! that files share no definitions.  Each `define-hazard` form in it records
//! the signature of a hazard as data and turns its body into a procedure,
//! which runs once each time the hazard is computed.  The body usually
//! returns a raster built by `pixelwise`, whose expression is then evaluated
//! natively at every pixel, so no Scheme code runs per pixel.

mod conversion;
mod primitives;
mod procedure;

use std::{fs, path::Path, rc::Rc};

use anyhow::{Context, Result, anyhow, bail};
use itertools::Itertools;
use steel::{SteelErr, SteelVal, steel_vm::engine::Engine};

use crate::{hazard::Hazard, sexp::Sexp};
use procedure::{Environment, Procedure};

const PRELUDE: &str = include_str!("script/prelude.scm");

/// Hazards defined by the Scheme `source`, which `origin` names in errors.
pub fn load(source: &str, origin: &str) -> Result<Vec<Hazard>> {
    load_with(origin, |engine| engine.run(source.to_owned()))
}

/// Hazards defined by the Scheme file at `path`, which may `require` other
/// files relative to its own location.
pub fn load_file(path: &Path) -> Result<Vec<Hazard>> {
    let source =
        fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    // Byte order mark that some editors write first, which Steel reads as code
    let source = source
        .strip_prefix('\u{feff}')
        .map(str::to_owned)
        .unwrap_or(source);
    load_with(&path.display().to_string(), |engine| {
        engine.compile_and_run_raw_program_with_path(source, path.to_owned())
    })
}

/// Hazards defined by the program that `run` runs in a new engine.
fn load_with(
    origin: &str,
    run: impl FnOnce(&mut Engine) -> steel::rvals::Result<Vec<SteelVal>>,
) -> Result<Vec<Hazard>> {
    let load = || -> Result<Vec<Hazard>> {
        let mut engine = engine()?;
        run(&mut engine).map_err(|error| render(&engine, origin, &error))?;
        let definitions = definitions(&engine)?;
        let environment = Rc::new(Environment::new(engine, origin));
        let hazards = definitions
            .into_iter()
            .map(|(definition, procedure)| {
                let body = Procedure::new(procedure, Rc::clone(&environment));
                Hazard::parse(&definition, Box::new(body))
            })
            .collect::<Result<Vec<_>>>()?;
        if let Some(name) = hazards.iter().map(Hazard::name).duplicates().next() {
            bail!("hazard `{name}` is defined more than once");
        }
        Ok(hazards)
    };
    load().with_context(|| format!("cannot load hazards from {origin}"))
}

/// Steel engine with the procedures of geoai and its prelude.
///
/// Every file gets a new engine, since clones of an engine share their
/// compiler, and with it their macros and the modules they have required.
fn engine() -> Result<Engine> {
    let mut engine = Engine::new();
    primitives::register(&mut engine);
    engine
        .run(PRELUDE)
        .map_err(|error| render(&engine, "prelude.scm", &error))?;
    Ok(engine)
}

/// Hazards recorded by `define-hazard` while `engine` ran, each as its
/// signature and declarations with the procedure that computes it.
fn definitions(engine: &Engine) -> Result<Vec<(Sexp, SteelVal)>> {
    let recorded = engine
        .extract_value("%hazards")
        .map_err(|error| anyhow!("{error}"))?;
    let SteelVal::ListV(recorded) = recorded else {
        bail!("expected a list of hazards, found {recorded}");
    };
    recorded
        .iter()
        .map(|hazard| {
            let pair = match hazard {
                SteelVal::ListV(items) => items.iter().collect_array(),
                _ => None,
            };
            let [definition, procedure] = pair.with_context(|| {
                format!("expected a definition and a procedure, found {hazard}")
            })?;
            Ok((Sexp::try_from(definition)?, procedure.clone()))
        })
        .collect()
}

/// Steel error shown with the line that caused it, from whichever file of
/// `engine` that line is in.  A program run without a path is named
/// `origin`.
fn render(engine: &Engine, origin: &str, error: &SteelErr) -> anyhow::Error {
    let source = error
        .span()
        .and_then(|span| span.source_id())
        .and_then(|id| {
            let name = engine
                .get_path_for_source_id(&id)
                .map_or_else(|| origin.to_owned(), |path| path.display().to_string());
            Some((name, engine.get_source(&id)?))
        });
    let message = match source {
        Some((name, text)) => error.emit_result_to_string(&name, &text),
        None => error.to_string(),
    };
    anyhow!("{}", message.trim_end())
}
