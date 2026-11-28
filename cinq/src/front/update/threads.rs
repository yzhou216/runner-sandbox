// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Running blocking work off the interface thread

use std::thread;

use iced::{Task, futures::channel::oneshot};

/// Runs blocking `work` on a thread of its own once iced runs the task: off
/// the interface thread, where GO and Panic must never wait, and off the
/// workers of iced's executor, which keep the clock ticking.  The task gives
/// what the work returns, or nothing when the program closes first.
pub(super) fn on_own_thread<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> Task<T> {
    Task::future(async move {
        let (sender, receiver) = oneshot::channel();
        thread::spawn(move || {
            // Receiver has gone only when the program is closing
            let _ = sender.send(work());
        });
        receiver.await.ok()
    })
    .and_then(Task::done)
}
