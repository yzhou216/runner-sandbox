// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Context, Result, bail, ensure};
use itertools::Itertools;

use super::{Body, Hazard, Input, Kind, SampleType};
use crate::sexp::Sexp;

impl Hazard {
    /// Hazard computed by `body`, with the signature and declarations of
    /// `definition`, of the form `((name input ...) declaration ...)`.
    ///
    /// Each input is written `name` for a raster or `(name kind)` for another
    /// kind of value, and `(optional name)` or `(optional name kind)` if it
    /// may be left unbound.  The declarations `(type u8)` or `(type f32)`,
    /// `(description text)`, and `(nodata number)` are optional, and each may
    /// appear at most once.
    pub fn parse(definition: &Sexp, body: Box<dyn Body>) -> Result<Self> {
        let Some((signature, declarations)) = definition.as_list().and_then(<[Sexp]>::split_first)
        else {
            bail!("expected a signature and declarations, found {definition}");
        };
        let Some((name, inputs)) = signature.as_list().and_then(<[Sexp]>::split_first) else {
            bail!("expected a signature (name input ...), found {signature}");
        };
        let name = name
            .as_symbol()
            .with_context(|| format!("expected a hazard name, found {name}"))?;
        Self::from_parts(name, inputs, declarations, body)
            .with_context(|| format!("in hazard `{name}`"))
    }

    fn from_parts(
        name: &str,
        inputs: &[Sexp],
        declarations: &[Sexp],
        body: Box<dyn Body>,
    ) -> Result<Self> {
        let inputs = inputs.iter().map(input).collect::<Result<Vec<_>>>()?;
        if let Some(name) = inputs.iter().map(|input| &input.name).duplicates().next() {
            bail!("input `{name}` is declared more than once");
        }
        let keywords = declarations
            .iter()
            .filter_map(|declaration| declaration.as_list()?.first()?.as_symbol());
        if let Some(keyword) = keywords.duplicates().next() {
            bail!("`{keyword}` is declared more than once");
        }
        let hazard = Self {
            name: name.to_owned(),
            inputs,
            sample_type: SampleType::default(),
            description: None,
            nodata: None,
            body,
        };
        let hazard = declarations.iter().try_fold(hazard, Self::declare)?;
        if let Some(nodata) = hazard.nodata {
            ensure!(
                hazard.sample_type.can_hold(nodata),
                "nodata {nodata} does not fit in {}",
                hazard.sample_type
            );
        }
        Ok(hazard)
    }

    /// Apply a declaration such as `(type u8)`.
    fn declare(self, declaration: &Sexp) -> Result<Self> {
        let Some([Sexp::Symbol(keyword), argument]) = declaration.as_list() else {
            bail!("expected a declaration such as (type u8), found {declaration}");
        };
        let expected = |kind: &str| format!("expected {kind} in {declaration}");
        Ok(match keyword.as_str() {
            "type" => Self {
                sample_type: argument
                    .as_symbol()
                    .with_context(|| expected("u8 or f32"))?
                    .parse()?,
                ..self
            },
            "description" => Self {
                description: Some(
                    argument
                        .as_string()
                        .with_context(|| expected("a string"))?
                        .to_owned(),
                ),
                ..self
            },
            "nodata" => Self {
                nodata: Some(argument.as_number().with_context(|| expected("a number"))?),
                ..self
            },
            _ => bail!("unknown declaration {declaration}"),
        })
    }
}

/// Input written as `name`, `(name kind)`, `(optional name)`, or
/// `(optional name kind)`.
fn input(sexp: &Sexp) -> Result<Input> {
    let is_optional = |keyword: &Sexp| keyword.as_symbol() == Some("optional");
    let (optional, name, kind) = match sexp.as_list() {
        None => (false, sexp, None),
        Some([keyword, name]) if is_optional(keyword) => (true, name, None),
        Some([keyword, name, kind]) if is_optional(keyword) => (true, name, Some(kind)),
        Some([name, kind]) => (false, name, Some(kind)),
        Some(_) => bail!(
            "expected an input such as pga, (threshold number), or (optional land), found {sexp}"
        ),
    };
    let name = name
        .as_symbol()
        .with_context(|| format!("expected an input name, found {name}"))?;
    let kind = match kind {
        Some(kind) => kind
            .as_symbol()
            .with_context(|| format!("expected an input kind, found {kind}"))?
            .parse()?,
        None => Kind::Raster,
    };
    Ok(Input {
        name: name.to_owned(),
        kind,
        optional,
    })
}
