// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! List of the network interfaces, read from the system at most once a
//! second, since reading it is a system call that is slow on some systems
//! while the sender runs a round for every frame of a fade

use std::time::{Duration, Instant};

const AGE: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub(super) struct Listing<T> {
    items: Vec<T>,
    read_at: Option<Instant>,
}

impl<T> Listing<T> {
    pub(super) fn new() -> Self {
        Self {
            items: Vec::new(),
            read_at: None,
        }
    }

    /// Returns the list as of `now`, read again with `read` only when the
    /// last reading is a second old, or none has been made
    pub(super) fn at(self, now: Instant, read: impl FnOnce() -> Vec<T>) -> Self {
        match self.read_at {
            Some(at) if now.saturating_duration_since(at) < AGE => self,
            _ => Self {
                items: read(),
                read_at: Some(now),
            },
        }
    }

    pub(super) fn items(&self) -> &[T] {
        &self.items
    }
}
