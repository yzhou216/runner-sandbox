// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Update cycle, which applies each message to the interface state
//!
//! Only [`cinq`] sends commands to the back-end outputs, which are the
//! sound, video, light, and network.  Showing, links, loading, and the
//! audio device also spawn threads or touch files, each at a boundary of
//! its own, and every other module only changes the state.

mod cinq;
mod clock;
mod device;
mod dispatch;
mod editing;
mod focus;
mod handling;
mod hints;
mod keys;
mod link;
mod loading;
mod outputs;
mod remote;
mod rows;
mod search;
mod settled;
mod show;
mod stage;
mod subscription;
#[cfg(test)]
mod tests;
mod threads;

use super::{App, Message};
pub(super) use editing::{Clipboard, Clipped};
pub(super) use hints::Hints;
use link::{edit_config, open_link, open_tutorial};
pub(super) use rows::{History, Measured, Measures, Rows};
pub(super) use search::{Search, Searched};
pub(super) use show::{Keeper, Read, Reader, Saved, Show};
pub(super) use stage::Stage;
use threads::on_own_thread;
