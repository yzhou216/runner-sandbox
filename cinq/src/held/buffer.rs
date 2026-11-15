// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The bytes that wait for the main screen, capped at the newest of them

use std::collections::VecDeque;

/// Most bytes that the buffer keeps
pub const CAP: usize = 1 << 20;

#[derive(Default)]
pub struct Buffer {
    /// Newest bytes written, oldest first
    bytes: VecDeque<u8>,
    /// Bytes that came before those and no longer fit
    dropped: usize,
}

impl Buffer {
    pub const fn new() -> Self {
        Self {
            bytes: VecDeque::new(),
            dropped: 0,
        }
    }

    /// Returns the buffer with `bytes` after those it held, and the oldest
    /// bytes dropped that no longer fit
    pub fn pushed(self, bytes: &[u8]) -> Self {
        let skipped = bytes.len().saturating_sub(CAP);
        // Queue that its API only extends and drains in place
        let mut queue = self.bytes;
        queue.extend(&bytes[skipped..]);
        let evicted = queue.len().saturating_sub(CAP);
        queue.drain(..evicted);
        Self {
            bytes: queue,
            dropped: self.dropped + skipped + evicted,
        }
    }

    /// Returns what to write once the main screen shows: a line that counts
    /// the bytes dropped, when any were, and then the bytes kept
    pub fn written(self) -> Vec<u8> {
        let notice = (self.dropped > 0).then(|| {
            format!(
                "cinq: {} bytes of output from while the problems showed were dropped\n",
                self.dropped
            )
        });
        notice
            .into_iter()
            .flat_map(String::into_bytes)
            .chain(self.bytes)
            .collect()
    }
}
