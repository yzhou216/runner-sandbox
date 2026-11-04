// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Event tap that reverses scroll events on their way through the login
//! session.

mod access;
mod cf;
mod error;
mod event;
mod hid;
mod registry;
mod sys;
mod tap;

use std::convert::Infallible;

pub use self::error::Error;
use self::tap::Tap;
use crate::reverser::Reverser;

impl Reverser {
    /// Reverses scroll events on the current run loop until the tap is invalidated.
    pub fn run(self) -> Result<Infallible, Error> {
        Tap::install(self)?;
        log!("version {}, reversing {self}", env!("CARGO_PKG_VERSION"));
        sys::CFRunLoopRun();
        Err(Error::Invalidated)
    }
}
