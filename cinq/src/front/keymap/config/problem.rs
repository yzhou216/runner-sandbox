// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Problems with the configuration file, and where in its text they lie
//!
//! The sidebar lists each problem after the name of the file and the line
//! that the problem lies on.  The watcher writes each to standard error as
//! miette draws it, with the lines of the file around it and the part that
//! it lies in marked, so that a terminal that cinq was started from shows
//! what is wrong with the file each time it is read.

#[cfg(test)]
mod tests;

use std::ops::Range;

use miette::{LabeledSpan, MietteDiagnostic, NamedSource, Report, Severity};

use super::Config;

/// Problem with the configuration file, which leaves the keys of the
/// keymap in place
#[derive(Debug, Clone)]
pub struct Problem {
    /// What is wrong, written to follow the name of the file
    pub message: String,
    /// Bytes of the text of the file that the problem lies in, if it lies
    /// in any
    pub span: Option<Range<usize>>,
}

impl Problem {
    /// Returns a problem that lies in no part of the text in particular
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    /// Returns the problem as lying in `span` of the text, or in no part of
    /// it without one
    pub fn at(self, span: impl Into<Option<Range<usize>>>) -> Self {
        Self {
            span: span.into(),
            ..self
        }
    }

    /// Returns the problem written after `place`, such as the keymap that
    /// it lies in
    pub fn after(self, place: &str) -> Self {
        Self {
            message: format!("{place}{}", self.message),
            ..self
        }
    }
}

impl Config {
    /// Returns each problem as the sidebar lists it: after the name of the
    /// file, and the line that it lies on when it lies in the text
    pub fn warnings(&self) -> impl Iterator<Item = String> + '_ {
        self.problems.iter().map(|problem| {
            let line = problem
                .span
                .as_ref()
                .zip(self.text.as_deref())
                .map_or_else(String::new, |(span, text)| format!(":{}", line(text, span)));
            format!("config.toml{line}: {}", problem.message)
        })
    }

    /// Returns a report of each problem for miette to draw: a warning, since
    /// the rest of the file applies, with the text of the file that the
    /// problem lies in
    pub fn reports(&self) -> impl Iterator<Item = Report> + '_ {
        let source = self
            .source
            .file()
            .zip(self.text.as_deref())
            .map(|(file, text)| {
                // Line break at the end, without which miette marks no place
                // past the last character of a file that lacks one
                let ended = if text.ends_with('\n') {
                    text.to_owned()
                } else {
                    format!("{text}\n")
                };
                NamedSource::new(file.display().to_string(), ended)
            });
        self.problems.iter().map(move |problem| {
            let diagnostic = MietteDiagnostic::new(format!("config.toml: {}", problem.message))
                .with_severity(Severity::Warning)
                .with_labels(problem.span.clone().map(LabeledSpan::underline));
            match source.clone() {
                Some(source) => Report::new(diagnostic).with_source_code(source),
                None => Report::new(diagnostic),
            }
        })
    }
}

/// Returns the line of `text` that `span` starts on, counted from one
fn line(text: &str, span: &Range<usize>) -> usize {
    text.bytes()
        .take(span.start)
        .filter(|&byte| byte == b'\n')
        .count()
        + 1
}
