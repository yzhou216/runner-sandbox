// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Mixer without a device, for tests that play through a mix that they
//! render

use rtrb::RingBuffer;

use super::{COMMANDS, Mix, Mixer, Tells};

impl Mixer {
    /// Returns a mixer without a device, whose ring of commands holds
    /// `capacity`, and the mix that the stream of a device would run
    pub(in crate::back::audio) fn bench(
        channels: u16,
        asked: u16,
        sample_rate: u32,
        capacity: usize,
    ) -> (Self, Mix) {
        let (commands, taken) = RingBuffer::new(capacity);
        let (handed, gone) = RingBuffer::new(COMMANDS);
        let mix = Mix::new(usize::from(channels), sample_rate, 8, taken, handed);
        let mixer = Self {
            _hold: None,
            commands,
            gone,
            sample_rate,
            channels,
            asked,
            tells: Tells::new(),
            next: 0,
        };
        (mixer, mix)
    }
}
