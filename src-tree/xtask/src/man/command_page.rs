// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! scrollreversed(1), rendered from the command definition.

use std::io;

use clap_mangen::Man;
use roff::{Roff, italic, roman};

use super::roff_ext::RoffExt;

/// Renders scrollreversed(1), adding the FILES and SEE ALSO sections that
/// clap knows nothing about.
pub fn render(config_path: &str, date: &str) -> io::Result<Vec<u8>> {
    let page = Man::new(scrollreversed::cli::command()).date(date);
    let mut out = Vec::new();
    page.render_title(&mut out)?;
    page.render_name_section(&mut out)?;
    page.render_synopsis_section(&mut out)?;
    page.render_description_section(&mut out)?;
    page.render_options_section(&mut out)?;
    Roff::new()
        .control("SH", ["FILES"])
        .control("TP", [])
        .text([italic(config_path)])
        .text([roman("Configuration file, read at startup.")])
        .see_also(&[("scrollreversed", "5"), ("launchctl", "1")])
        .to_writer(&mut out)?;
    page.render_version_section(&mut out)?;
    Ok(out)
}
