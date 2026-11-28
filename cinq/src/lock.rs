// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Locking a mutex whose holder may have panicked

use std::sync::{Mutex, MutexGuard, PoisonError};

/// Locks `mutex`, which a thread that panicked while it held the lock leaves
/// as it was
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
