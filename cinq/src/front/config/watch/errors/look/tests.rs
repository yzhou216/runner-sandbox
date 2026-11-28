// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how the problems are drawn, wherever standard error points

use std::fmt;

use miette::{MietteDiagnostic, ReportHandler, Severity};

use super::Look;

/// Returns a diagnostic as a handler of `look` draws it at `width`
fn drawn(look: Look, width: Option<u16>) -> String {
    struct Show<'a>(&'a dyn ReportHandler, &'a MietteDiagnostic);
    impl fmt::Display for Show<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.debug(self.1, f)
        }
    }
    let diagnostic = MietteDiagnostic::new(
        "config.toml: no setting is named `colour`, and none is like it either",
    )
    .with_severity(Severity::Warning);
    Show(&look.handler(width), &diagnostic).to_string()
}

const PLAIN: Look = Look {
    color: false,
    truecolor: false,
    links: false,
    unicode: false,
};

#[test]
fn colors_follow_the_terminal_that_was_asked_not_standard_error_now() {
    let colored = drawn(
        Look {
            color: true,
            ..PLAIN
        },
        None,
    );
    assert!(colored.contains("\x1b["), "{colored:?}");
    assert!(!drawn(PLAIN, None).contains('\x1b'));
}

#[test]
fn characters_follow_the_terminal_that_was_asked() {
    let unicode = drawn(
        Look {
            unicode: true,
            ..PLAIN
        },
        None,
    );
    assert!(!unicode.is_ascii(), "{unicode:?}");
    assert!(drawn(PLAIN, None).is_ascii());
}

#[test]
fn report_wraps_at_the_width_it_is_given() {
    let lines = |width| drawn(PLAIN, Some(width)).lines().count();
    assert!(lines(20) > lines(120));
}
