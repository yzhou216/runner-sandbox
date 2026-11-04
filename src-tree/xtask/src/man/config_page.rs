// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! scrollreversed(5), describing the configuration file.

use roff::{Roff, bold, italic, roman};
use scrollreversed::reverser::{Axes, Devices};

use super::roff_ext::RoffExt;

/// Renders scrollreversed(5), taking keys, choices, and defaults from the
/// types the daemon parses the file into.
pub fn render(config_path: &str, date: &str) -> Vec<u8> {
    let source = format!(
        "scrollreversed {}",
        scrollreversed::cli::command()
            .get_version()
            .unwrap_or_default()
    );
    Roff::new()
        .control(
            "TH",
            ["scrollreversed", "5", date, &source, "File Formats Manual"],
        )
        .control("SH", ["NAME"])
        .text([roman(
            "scrollreversed - configuration file for scrollreversed",
        )])
        .control("SH", ["SYNOPSIS"])
        .text([italic(config_path)])
        .control("SH", ["DESCRIPTION"])
        .text([
            bold("scrollreversed"),
            roman(
                "(1) reads its settings at startup from a TOML file in the \
                 user's Application Support directory.  The ",
            ),
            bold("--config"),
            roman(" option names a different file."),
        ])
        .control("PP", [])
        .text([roman(
            "Every key is optional, and a missing file leaves every setting at \
             its default.  Unknown keys and values are errors.  Options given \
             on the command line override the file.  Changes take effect when \
             scrollreversed restarts.",
        )])
        .control("SH", ["KEYS"])
        .choice_key::<Devices>("device", "Device whose scrolling is reversed.")
        .choice_key::<Axes>("axis", "Axis along which scrolling is reversed.")
        .control("TP", [])
        .text([bold("verbose"), roman(" = "), italic("boolean")])
        .text([roman(
            "Log every scroll event and how it was classified to standard \
             error.  Defaults to false.",
        )])
        .control("SH", ["EXAMPLES"])
        .text([roman(
            "Reverse only vertical trackpad scrolling, for natural scrolling \
             turned off in System Settings:",
        )])
        .example(&["device = \"trackpad\"", "axis = \"vertical\""])
        .control("PP", [])
        .text([roman("Restart the launchd agent to apply the change:")])
        .example(&["launchctl kickstart -k gui/$(id -u)/io.yiyuzhou.scrollreversed"])
        .see_also(&[("scrollreversed", "1"), ("launchctl", "1")])
        .render()
        .into_bytes()
}
