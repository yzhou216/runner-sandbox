// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Darwin daemon reversing mouse or trackpad scrolling

/// Prints a line to standard error, prefixed with the program name
#[cfg(target_os = "macos")]
macro_rules! log {
    ($($arg:tt)*) => {
        eprintln!("scrollreversed: {}", format_args!($($arg)*))
    };
}

pub mod cli;
pub mod config;
#[cfg(target_os = "macos")]
mod macos;
pub mod reverser;

#[cfg(target_os = "macos")]
pub use macos::Error;
