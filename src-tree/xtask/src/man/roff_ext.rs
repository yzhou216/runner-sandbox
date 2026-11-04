// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Manual page building blocks on top of the roff builder.

use clap::ValueEnum;
use roff::{Roff, bold, italic, roman};

/// Chainable helpers for the sections the manual pages share
pub trait RoffExt {
    /// Documents a configuration key whose value is one of the choices of
    /// `T`, with each choice's help text and the default.
    fn choice_key<T>(&mut self, key: &str, summary: &str) -> &mut Self
    where
        T: ValueEnum + Default;

    /// Shows `lines` as an indented, unfilled block.
    fn example(&mut self, lines: &[&str]) -> &mut Self;

    /// Lists related manual pages as `(name, section)` pairs.
    fn see_also(&mut self, pages: &[(&str, &str)]) -> &mut Self;
}

impl RoffExt for Roff {
    fn choice_key<T>(&mut self, key: &str, summary: &str) -> &mut Self
    where
        T: ValueEnum + Default,
    {
        let default = T::default()
            .to_possible_value()
            .map_or_else(String::new, |value| value.get_name().to_owned());
        T::value_variants()
            .iter()
            .filter_map(ValueEnum::to_possible_value)
            .fold(
                self.control("TP", [])
                    .text([bold(key), roman(" = "), italic("string")])
                    .text([roman(format!("{summary}  Defaults to \"{default}\"."))]),
                |roff, value| {
                    roff.control("RS", [])
                        .control("TP", [])
                        .text([bold(format!("\"{}\"", value.get_name()))])
                        .text([roman(
                            value
                                .get_help()
                                .map_or_else(String::new, ToString::to_string),
                        )])
                        .control("RE", [])
                },
            )
    }

    fn example(&mut self, lines: &[&str]) -> &mut Self {
        lines
            .iter()
            .fold(
                self.control("PP", [])
                    .control("nf", [])
                    .control("RS", ["4"]),
                |roff, line| roff.text([roman(*line)]),
            )
            .control("RE", [])
            .control("fi", [])
    }

    fn see_also(&mut self, pages: &[(&str, &str)]) -> &mut Self {
        let references = pages
            .iter()
            .enumerate()
            .flat_map(|(index, (name, section))| {
                let separator = if index + 1 < pages.len() { ", " } else { "" };
                [bold(*name), roman(format!("({section}){separator}"))]
            })
            .collect::<Vec<_>>();
        self.control("SH", ["SEE ALSO"]).text(references)
    }
}
