// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Location of the configuration file.

use std::path::PathBuf;

use etcetera::base_strategy::{Apple, BaseStrategy};

const FILE: &str = "scrollreversed/config.toml";

/// Returns `~/Library/Application Support/scrollreversed/config.toml`.
///
/// Darwin leaves Preferences to `defaults` and keeps other application files
/// in Application Support, etcetera's data directory.  The Apple strategy is
/// explicit so that manual pages generated on any host show this path.
pub fn default_path() -> Option<PathBuf> {
    Apple::new().ok().map(|apple| apple.data_dir().join(FILE))
}

/// Returns the default location with the home directory written as `~`,
/// for the help text and the manual pages.
pub fn documented_path() -> Option<String> {
    let apple = Apple::new().ok()?;
    apple
        .data_dir()
        .join(FILE)
        .strip_prefix(apple.home_dir())
        .ok()
        .map(|path| format!("~/{}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_follow_the_apple_convention() {
        assert!(default_path().is_none_or(|path| {
            path.ends_with("Library/Application Support/scrollreversed/config.toml")
        }));
        assert!(
            documented_path().is_none_or(
                |path| path == "~/Library/Application Support/scrollreversed/config.toml"
            )
        );
    }
}
