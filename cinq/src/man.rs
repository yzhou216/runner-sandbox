// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Manual pages, which the build script renders for the platform that it
//! builds for
//!
//! cinq(1) comes from the command line, which clap_mangen renders.  cinq(5)
//! and cinq-show(5) come from their scdoc sources, of which the build keeps
//! only the parts for the platform, for scdoc to render in turn.  The
//! program itself leaves this module out: only the build script and the
//! tests compile it.

mod command;
#[cfg(test)]
mod tests;

use strum::{IntoStaticStr, VariantArray};

pub use command::command_page;

pub const SCDOC: [(&str, &str); 2] = [
    (
        "cinq.5",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/man/cinq.5.scd")),
    ),
    (
        "cinq-show.5",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/man/cinq-show.5.scd")),
    ),
];

/// Line of a source in scdoc that starts a part for the platforms named
/// after it, which lasts until the next such line or [`END`]
const IF: &str = "; if ";

/// Line of a source in scdoc that ends a part for some platforms, after
/// which the source is for every platform again
const END: &str = "; end";

/// Platform, as far as the manual pages tell platforms apart: by where the
/// dirs crate finds the configuration directory
#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantArray, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum Platform {
    /// POSIX systems that follow the XDG Base Directory Specification, such
    /// as GNU/Linux and the BSDs
    Xdg,
    Darwin,
    Windows,
}

impl Platform {
    pub fn of(target_os: &str) -> Self {
        match target_os {
            "windows" => Platform::Windows,
            "macos" | "ios" => Platform::Darwin,
            _ => Platform::Xdg,
        }
    }

    /// Returns the name that marks the parts of the scdoc sources for the
    /// platform
    fn name(self) -> &'static str {
        self.into()
    }

    fn config_file(self) -> &'static str {
        match self {
            Platform::Xdg => "$XDG_CONFIG_HOME/cinq/config.toml",
            Platform::Darwin => "~/Library/Application Support/cinq/config.toml",
            Platform::Windows => r"%APPDATA%\cinq\config.toml",
        }
    }

    fn unsaved_directory(self) -> &'static str {
        match self {
            Platform::Xdg => "$XDG_STATE_HOME/cinq/unsaved",
            Platform::Darwin => "~/Library/Application Support/cinq/unsaved",
            Platform::Windows => r"%LOCALAPPDATA%\cinq\unsaved",
        }
    }
}

/// Returns the scdoc `source` of a page for `platform`, without the parts
/// for other platforms, and without the lines that mark the parts
pub fn for_platform(source: &str, platform: Platform) -> String {
    source
        .lines()
        .scan(true, |kept, line| {
            Some(match line.strip_prefix(IF) {
                Some(platforms) => {
                    *kept = platforms
                        .split_whitespace()
                        .any(|name| name == platform.name());
                    None
                }
                None if line == END => {
                    *kept = true;
                    None
                }
                None => kept.then_some(line),
            })
        })
        .flatten()
        .flat_map(|line| [line, "\n"])
        .collect()
}
