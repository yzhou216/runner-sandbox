// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Command-line interface, also the source of scrollreversed(1).

use std::path::{Path, PathBuf};

use clap::{CommandFactory, Parser};

use crate::{
    config,
    reverser::{Axes, Devices, Settings},
};

/// scrollreversed reverses the scroll direction of either the mouse or the
/// trackpad, so that each can scroll its own way despite the single natural
/// scrolling setting in System Settings.
///
/// Settings come from the configuration file described in
/// scrollreversed(5).  Options given on the command line override them.
///
/// The device behind each scroll event is looked up in the I/O Registry and
/// classified the way Darwin picks between the Mouse and Trackpad scroll
/// settings, so a Magic Mouse counts as a mouse.  Events whose sender is not
/// in the registry, such as events posted by other applications or
/// forwarded by Universal Control, are classified by gesture phase instead.
///
/// The process needs Accessibility access.  Without it, scrollreversed asks
/// for access once and waits until it is granted under System Settings >
/// Privacy & Security > Accessibility.  Darwin only honours the grant in a
/// new process, so scrollreversed then exits for launchd to restart it.
#[derive(Debug, Parser)]
#[command(
    name = "scrollreversed",
    version,
    about = "Darwin daemon reversing mouse or trackpad scrolling"
)]
pub struct Args {
    /// Device whose scrolling is reversed, overriding the device key of the
    /// configuration file [default: mouse]
    #[arg(short, long, value_enum)]
    device: Option<Devices>,

    /// Axis along which scrolling is reversed, overriding the axis key of the
    /// configuration file [default: both]
    #[arg(short, long, value_enum)]
    axis: Option<Axes>,

    /// Log every scroll event and how it was classified to standard error
    #[arg(short, long)]
    verbose: bool,

    // Computed to show the actual default path
    #[arg(short, long, value_name = "PATH", help = config_help())]
    config: Option<PathBuf>,
}

impl Args {
    pub fn config(&self) -> Option<&Path> {
        self.config.as_deref()
    }

    /// Returns the options as settings, where `--verbose` only turns logging on.
    pub fn settings(&self) -> Settings {
        Settings {
            device: self.device,
            axis: self.axis,
            verbose: self.verbose.then_some(true),
        }
    }
}

fn config_help() -> String {
    let default = config::documented_path().unwrap_or_else(|| "the default file".to_owned());
    format!("Read the configuration from PATH, which must exist, instead of {default}")
}

/// Returns the command definition, from which scrollreversed(1) is rendered.
pub fn command() -> clap::Command {
    Args::command()
}

#[cfg(test)]
mod tests {
    use clap::error::ErrorKind;

    use super::*;

    fn parse(args: &[&str]) -> Result<Args, ErrorKind> {
        Args::try_parse_from(std::iter::once("scrollreversed").chain(args.iter().copied()))
            .map_err(|error| error.kind())
    }

    fn settings(args: &[&str]) -> Result<Settings, ErrorKind> {
        parse(args).map(|args| args.settings())
    }

    #[test]
    fn definition_is_consistent() {
        command().debug_assert();
    }

    #[test]
    fn no_arguments_give_no_settings() {
        assert_eq!(settings(&[]), Ok(Settings::default()));
    }

    #[test]
    fn options_take_separate_or_inline_values() {
        let expected = Settings {
            device: Some(Devices::Trackpad),
            axis: Some(Axes::Vertical),
            verbose: Some(true),
        };
        assert_eq!(
            settings(&["--device", "trackpad", "--axis", "vertical", "--verbose"]),
            Ok(expected)
        );
        assert_eq!(
            settings(&["--device=trackpad", "-a", "vertical", "-v"]),
            Ok(expected)
        );
    }

    #[test]
    fn config_help_names_the_default_path() {
        let help = command()
            .get_arguments()
            .find(|arg| arg.get_id() == "config")
            .and_then(|arg| arg.get_help())
            .map(ToString::to_string);
        let path = config::documented_path();
        assert!(help.is_some_and(|help| path.is_none_or(|path| help.contains(&path))));
    }

    #[test]
    fn config_path_is_kept() {
        assert_eq!(
            parse(&["-c", "/tmp/x.toml"]).map(|args| args.config().map(PathBuf::from)),
            Ok(Some(PathBuf::from("/tmp/x.toml")))
        );
    }

    #[test]
    fn invalid_arguments_are_rejected() {
        [
            (&["--device"][..], ErrorKind::InvalidValue),
            (&["--axis", "diagonal"], ErrorKind::InvalidValue),
            (&["mouse"], ErrorKind::UnknownArgument),
        ]
        .into_iter()
        .for_each(|(args, kind)| assert_eq!(settings(args).err(), Some(kind), "{args:?}"));
    }
}
