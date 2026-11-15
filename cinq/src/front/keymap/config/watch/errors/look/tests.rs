// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of how the problems are drawn, wherever standard error points

use std::fmt;

use miette::{MietteDiagnostic, ReportHandler, Severity};

use super::Look;

/// Diagnostic as a handler of `look` draws it at `width`
struct Drawn(String);

impl Drawn {
    fn new(look: Look, width: Option<u16>) -> Self {
        let diagnostic = MietteDiagnostic::new(
            "config.toml: no setting is named `colour`, and none is like it either",
        )
        .with_severity(Severity::Warning);
        struct Show<'a>(&'a dyn ReportHandler, &'a MietteDiagnostic);
        impl fmt::Display for Show<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.debug(self.1, f)
            }
        }
        Self(Show(&look.handler(width), &diagnostic).to_string())
    }
}

const PLAIN: Look = Look {
    color: false,
    truecolor: false,
    links: false,
    unicode: false,
};

#[test]
fn colors_follow_the_terminal_that_was_asked_not_standard_error_now() {
    let colored = Drawn::new(
        Look {
            color: true,
            ..PLAIN
        },
        None,
    );
    assert!(colored.0.contains("\x1b["), "{:?}", colored.0);
    assert!(!Drawn::new(PLAIN, None).0.contains('\x1b'));
}

#[test]
fn characters_follow_the_terminal_that_was_asked() {
    let unicode = Drawn::new(
        Look {
            unicode: true,
            ..PLAIN
        },
        None,
    );
    assert!(!unicode.0.is_ascii(), "{:?}", unicode.0);
    assert!(Drawn::new(PLAIN, None).0.is_ascii());
}

#[test]
fn report_wraps_at_the_width_it_is_given() {
    let lines = |width| Drawn::new(PLAIN, Some(width)).0.lines().count();
    assert!(lines(20) > lines(120));
}
