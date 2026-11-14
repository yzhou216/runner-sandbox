// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Backend of kira that hands its renderer to the mixer of a device
//!
//! kira mixes every sound of a manager in stereo, and writes silence to the
//! channels of a device past the second.  Each sound therefore plays in a
//! manager of its own, whose renderer the mixer of the device takes, so
//! that the mixer can send the left and right of each sound to the channels
//! of the device that its outputs name.

use std::convert::Infallible;

use kira::backend::{Backend, Renderer};

pub(super) struct Feed {
    renderer: Option<Renderer>,
}

impl Feed {
    /// Returns the renderer, which kira hands over as the manager opens
    pub(super) fn take(&mut self) -> Option<Renderer> {
        self.renderer.take()
    }
}

impl Backend for Feed {
    /// Sample rate of the device that the sound plays on
    type Settings = u32;
    type Error = Infallible;

    fn setup(sample_rate: u32, _internal_buffer_size: usize) -> Result<(Self, u32), Infallible> {
        Ok((Self { renderer: None }, sample_rate))
    }

    fn start(&mut self, renderer: Renderer) -> Result<(), Infallible> {
        self.renderer = Some(renderer);
        Ok(())
    }
}
