// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Problems with the configuration file, and where in its text they lie
//!
//! The sidebar lists them, and the watcher writes them to standard error as
//! miette draws them, so that a terminal that cinq was started from shows
//! what is wrong with the file each time it is read.

#[cfg(test)]
mod tests;

use std::{ops::Range, path::Path};

use miette::{LabeledSpan, MietteDiagnostic, NamedSource, Report, Severity};

use super::Config;

#[derive(Debug, Clone)]
pub struct Problem {
    /// What is wrong, written to follow the name of the file
    pub message: String,
    /// Bytes of the text of the file that the problem lies in, if it lies
    /// in any
    pub span: Option<Range<usize>>,
}

impl Problem {
    pub(super) fn new(message: impl Into<String>) -> Self {
        Self::at(None, message)
    }

    pub(super) fn at(span: impl Into<Option<Range<usize>>>, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: span.into(),
        }
    }
}

impl Config {
    /// Returns each problem as the sidebar lists it
    pub fn warnings(&self) -> impl Iterator<Item = String> + '_ {
        let text = self.text.as_deref().unwrap_or_default();
        self.problems
            .iter()
            .map(move |problem| match &problem.span {
                Some(span) => format!("config.toml:{}: {}", line(text, span), problem.message),
                None => format!("config.toml: {}", problem.message),
            })
    }

    /// Each report is a warning, since the rest of the file applies.
    pub(super) fn reports(&self) -> impl Iterator<Item = Report> + '_ {
        let file = self.source.file().unwrap_or(Path::new("config.toml"));
        let text = self.text.as_deref().unwrap_or_default();
        // Line break at the end, without which miette marks no place past
        // the last character of a file that lacks one
        let ended = format!("{}\n", text.strip_suffix('\n').unwrap_or(text));
        let source = NamedSource::new(file.display().to_string(), ended);
        self.problems.iter().map(move |problem| {
            let diagnostic = MietteDiagnostic::new(format!("config.toml: {}", problem.message))
                .with_severity(Severity::Warning)
                .with_labels(problem.span.clone().map(LabeledSpan::underline));
            Report::new(diagnostic).with_source_code(source.clone())
        })
    }
}

/// Returns the line of `text` that `span` starts on, counted from one
fn line(text: &str, span: &Range<usize>) -> usize {
    text[..span.start].matches('\n').count() + 1
}
