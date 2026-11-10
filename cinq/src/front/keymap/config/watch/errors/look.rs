// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! How miette draws the problems, settled before output is held
//!
//! Each time miette draws a report, it asks standard error whether it is a
//! terminal and what such a terminal takes: colors, links, and Unicode.
//! While output is held, standard error is a pipe, and the problems would
//! come out plain.  So the answers are asked once, when the watch of a
//! terminal starts, and the width of the terminal each time a report is
//! drawn, from the terminal itself.

#[cfg(test)]
mod tests;

use std::sync::Once;

use miette::{MietteHandler, MietteHandlerOpts, RgbColors};
use supports_color::Stream;
use terminal_size::{Width, terminal_size_of};

use crate::held;

/// What the terminal on standard error takes of what miette draws
#[derive(Clone, Copy)]
struct Look {
    color: bool,
    /// Whether colors are those of sixteen million, not of a palette
    truecolor: bool,
    links: bool,
    unicode: bool,
}

impl Look {
    fn detected() -> Self {
        let color = supports_color::on(Stream::Stderr);
        Self {
            color: color.is_some(),
            truecolor: color.is_some_and(|level| level.has_16m),
            links: supports_hyperlinks::on(supports_hyperlinks::Stream::Stderr),
            unicode: supports_unicode::on(supports_unicode::Stream::Stderr),
        }
    }

    /// Returns the handler that draws at `width` columns, or at the width
    /// that miette finds when there is none
    fn handler(self, width: Option<u16>) -> MietteHandler {
        let opts = MietteHandlerOpts::new()
            .color(self.color)
            .rgb_colors(if self.truecolor {
                RgbColors::Always
            } else {
                RgbColors::Never
            })
            .terminal_links(self.links)
            .unicode(self.unicode);
        match width {
            Some(width) => opts.width(usize::from(width)),
            None => opts,
        }
        .build()
    }
}

/// Makes miette draw as the terminal on standard error takes, wherever
/// standard error points when a report is drawn
pub fn install() {
    static INSTALLED: Once = Once::new();
    INSTALLED.call_once(|| {
        let look = Look::detected();
        // Only a hook set before makes this fail, and none is
        let _ = miette::set_hook(Box::new(move |_| {
            let size = held::terminal()
                .ok()
                .and_then(|terminal| terminal_size_of(&terminal));
            Box::new(look.handler(size.map(|(Width(columns), _)| columns)))
        }));
    });
}
